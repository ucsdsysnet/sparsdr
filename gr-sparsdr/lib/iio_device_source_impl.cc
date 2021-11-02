/* -*- c++ -*- */
/*
 * Copyright 2021 The Regents of the University of California.
 *
 * This is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 3, or (at your option)
 * any later version.
 *
 * This software is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this software; see the file COPYING.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street,
 * Boston, MA 02110-1301, USA.
 */

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "iio_device_source_impl.h"
#include <gnuradio/io_signature.h>

namespace gr {
namespace sparsdr {

iio_device_source::sptr iio_device_source::make(iio_device* device,
                                                const std::string& channel,
                                                std::size_t buffer_size_samples)
{
    return gnuradio::get_initial_sptr(
        new iio_device_source_impl(device, channel, buffer_size_samples));
}

/*
 * The private constructor
 */
iio_device_source_impl::iio_device_source_impl(iio_device* device,
                                               const std::string& channel,
                                               std::size_t buffer_size_samples)
    : gr::sync_block(
          "iio_device_source",
          gr::io_signature::make(0, 0, 0),
          // Output is in 4-byte chunks. The output type is not really important.
          gr::io_signature::make(1, 1, sizeof(uint32_t))),
      d_device(device),
      d_buffer(nullptr),
      d_channel(nullptr),
      d_buffer_size_samples(buffer_size_samples),
      d_buffer_mutex(),
      d_refill_cv(),
      d_samples_ready_cv(),
      d_refill_thread(),
      d_samples_in_buffer(0),
      d_sample_offset(0),
      d_please_refill_buffer(false),
      d_thread_stopped(false)
{
    // Disable all channels on the device
    for (unsigned int i = 0; i < iio_device_get_channels_count(device); i++) {
        iio_channel* const channel = iio_device_get_channel(device, i);
        iio_channel_disable(channel);
    }
    // Find and enable the desired channel
    d_channel = iio_device_find_channel(device, channel.c_str(), false);
    if (d_channel == nullptr) {
        throw std::runtime_error("Channel not found on device");
    }
    iio_channel_enable(d_channel);
}

bool iio_device_source_impl::start()
{
    std::unique_lock<std::mutex> lock(d_buffer_mutex);
    d_samples_in_buffer = 0;
    d_sample_offset = 0;
    d_please_refill_buffer = false;
    d_thread_stopped = false;

    d_buffer = iio_device_create_buffer(d_device, d_buffer_size_samples, false);
    if (d_buffer == nullptr) {
        throw std::runtime_error("Failed to create buffer");
    }
    if (iio_buffer_step(d_buffer) != 2) {
        throw std::runtime_error("IIO sample size (buffer step) is not 2 bytes");
    }
    // Start thread to refill the buffer
    d_refill_thread = std::thread(&iio_device_source_impl::refill_thread, this);

    return true;
}

bool iio_device_source_impl::stop()
{
    // Tell the refill thread to exit
    // iio_buffer_cancel() is thread-safe
    if (d_buffer != nullptr) {
        iio_buffer_cancel(d_buffer);
    }
    std::unique_lock<std::mutex> lock(d_buffer_mutex);
    d_please_refill_buffer = true;
    d_refill_cv.notify_all();
    lock.unlock();

    d_refill_thread.join();

    if (d_buffer != nullptr) {
        iio_buffer_destroy(d_buffer);
        d_buffer = nullptr;
    }
    return true;
}

/** This runs in a dedicated worker thread */
void iio_device_source_impl::refill_thread()
{
    std::unique_lock<std::mutex> lock(d_buffer_mutex);
    ssize_t status = 0;
    while (true) {
        d_refill_cv.wait(lock, [this] { return this->d_please_refill_buffer; });
        // Now the work thread has requested more samples
        d_please_refill_buffer = false;

        lock.unlock();
        status = iio_buffer_refill(d_buffer);
        lock.lock();

        if (status < 0) {
            break;
        }
        // Calculate the number of samples read
        d_samples_in_buffer = static_cast<std::size_t>(status) / 2;
        d_sample_offset = 0;
        // Notify the work thread that samples are available
        d_samples_ready_cv.notify_all();
    }
    // iio_buffer_refill() returned an error
    // EBADF is not really an error. It indicates that the buffer was cancelled.
    if (status != -EBADF) {
        char error_description[512];
        iio_strerror(-status, error_description, sizeof error_description);
        std::cerr << "Failed to refill buffer: " << error_description << "\n";
        if (-status == ETIMEDOUT) {
            std::cerr << "This is normally caused by overflow because the "
                         "threshold is too low or too many bins are unmasked.\n";
        }
    }

    d_thread_stopped = true;
    d_samples_ready_cv.notify_all();
}

/*
 * Our virtual destructor.
 */
iio_device_source_impl::~iio_device_source_impl()
{
    // The buffer (if any) was already destroyed in stop().
    // Don't destroy the IIO context
}

int iio_device_source_impl::work(int noutput_items,
                                 gr_vector_const_void_star&,
                                 gr_vector_void_star& output_items)
{
    // Reminder: noutput_items is in 4-byte units. One block output item
    // equals two IIO samples.

    std::unique_lock<std::mutex> lock(d_buffer_mutex);
    if (d_thread_stopped) {
        // Can't read any more samples
        return -1;
    }
    // If the buffer is empty, ask the refill thread for more samples
    if (!d_please_refill_buffer && d_samples_in_buffer == d_sample_offset) {
        d_please_refill_buffer = true;
        d_refill_cv.notify_all();
    }
    // Wait for samples
    while (d_please_refill_buffer) {
        // This is the only part that actually requires the separate thread and
        // condition variables: using a timed wait, this code can detect
        // overflow if no samples appear within some time limit.
        d_samples_ready_cv.wait(lock);
        if (d_thread_stopped) {
            // Can' read any more samples
            return -1;
        }
    }

    const std::size_t samples_to_copy =
        std::min(d_samples_in_buffer - d_sample_offset, std::size_t(noutput_items) * 2);

    const void* const buffer_region_start = reinterpret_cast<const void*>(
        reinterpret_cast<const char*>(iio_buffer_start(d_buffer)) +
        (2 * d_sample_offset));
    std::memcpy(output_items[0], buffer_region_start, samples_to_copy * 2);

    d_sample_offset += samples_to_copy;

    // Tell runtime system how many output items we produced.
    // (convert back from 16-bit samples to 32-bit samples)
    return int(samples_to_copy / 2);
}

} /* namespace sparsdr */
} /* namespace gr */
