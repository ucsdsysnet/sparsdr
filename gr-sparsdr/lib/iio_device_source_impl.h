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

#ifndef INCLUDED_SPARSDR_IIO_DEVICE_SOURCE_IMPL_H
#define INCLUDED_SPARSDR_IIO_DEVICE_SOURCE_IMPL_H

#include <sparsdr/iio_device_source.h>

#include <condition_variable>
#include <iio.h>
#include <mutex>
#include <thread>

namespace gr {
namespace sparsdr {

class iio_device_source_impl : public iio_device_source
{
private:
    /** cf-ad9361-lpc IIO device */
    iio_device* d_device;
    /** Buffer used to read samples from the radio */
    iio_buffer* d_buffer;
    /** Channel used to read samples from the radio */
    iio_channel* d_channel;

    /** Mutex used to lock d_buffer */
    std::mutex d_buffer_mutex;
    /**
     * Condition variable used with d_buffer_mutex to notify the refill thread
     * when it should call iio_buffer_refill() again
     */
    std::condition_variable d_refill_cv;
    /**
     * Condition variable used with d_buffer_mutex to notify the work thread
     * when the refill thread has finished reading samples
     */
    std::condition_variable d_samples_ready_cv;
    /** Thread that calls iio_buffer_refill */
    std::thread d_refill_thread;

    // The following four fields and d_buffer are protected by d_buffer_mutex
    std::size_t d_samples_in_buffer;
    /**
     * Offset from the beginning of the buffer to the first sample that has not
     * been copied into a GNU Radio block output buffer
     */
    std::size_t d_sample_offset;
    bool d_please_refill_buffer;
    bool d_thread_stopped;

    /** Runs in a dedicated thread ands calls iio_buffer_refill */
    void refill_thread();

public:
    iio_device_source_impl(iio_device* device, const std::string& channel);
    ~iio_device_source_impl();

    // Where all the action really happens
    int work(int noutput_items,
             gr_vector_const_void_star& input_items,
             gr_vector_void_star& output_items) override;

    virtual bool start() override;
    virtual bool stop() override;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_IIO_DEVICE_SOURCE_IMPL_H */
