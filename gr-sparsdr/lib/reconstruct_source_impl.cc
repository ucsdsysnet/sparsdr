/* -*- c++ -*- */
/*
 * Copyright 2022 The Regents of the University of California.
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

#include "reconstruct_source_impl.h"
#include <gnuradio/io_signature.h>
#include <utility>

namespace gr {
namespace sparsdr {

reconstruct_source::sptr
reconstruct_source::make(std::unique_ptr<output_context>&& context)
{
    return gnuradio::get_initial_sptr(new reconstruct_source_impl(std::move(context)));
}


/*
 * The private constructor
 */
reconstruct_source_impl::reconstruct_source_impl(
    std::unique_ptr<output_context>&& context)
    : gr::sync_block("reconstruct_source",
                     gr::io_signature::make(0, 0, 0),
                     gr::io_signature::make(1, 1, sizeof(gr_complex))),
      d_context(std::move(context))
{
}

/*
 * Our virtual destructor.
 */
reconstruct_source_impl::~reconstruct_source_impl() {}

int reconstruct_source_impl::work(int noutput_items,
                                  gr_vector_const_void_star&,
                                  gr_vector_void_star& output_items)
{
    void* raw_out_buffer = output_items.at(0);
    gr_complex* out_buffer = reinterpret_cast<gr_complex*>(raw_out_buffer);
    // Lock the mutex and copy some outputs
    std::unique_lock<std::mutex> lock(d_context->mutex);

    // Wait until this output's queue of samples is not empty, or
    // the timeout has passed
    const bool got_samples = d_context->cv.wait_for(
        lock, std::chrono::seconds(1), [this] { return !d_context->queue.empty(); });

    // Copy one complex value at a time until the queue is empty or noutput_items
    // values have been copied
    int items_copied = 0;
    auto done = [this, &items_copied, noutput_items] {
        return items_copied == noutput_items || d_context->queue.empty();
    };
    for (items_copied = 0; !done(); items_copied++) {
        *out_buffer = d_context->queue.front();
        out_buffer += 1;
        d_context->queue.pop();
    }
    if (items_copied != 0) {
        std::cout << "Reconstruct sink produced" << items_copied << " items\n";
    }
    // Tell runtime system how many output items we produced.
    return items_copied;
}


// Callback that the reconstruction library calls when it has produced samples
void reconstruct_source_impl::handle_reconstructed_samples(
    void* context, const std::complex<float>* samples, std::size_t num_samples)
{
    try {
        output_context* output_context =
            reinterpret_cast<gr::sparsdr::output_context*>(context);
        // Lock the mutex and copy the samples to the queue
        std::unique_lock<std::mutex> lock(output_context->mutex);
        for (std::size_t i = 0; i < num_samples; i++) {
            output_context->queue.push(samples[i]);
        }
        lock.unlock();
        // Wake up the work thread that may be waiting for samples
        output_context->cv.notify_one();
    } catch (std::exception& e) {
        // Don't let C++ exceptions propagate into Rust
        std::cerr << "Unexpected exception in reconstructed sample callback: " << e.what()
                  << '\n';
        std::terminate();
    } catch (...) {
        std::cerr << "Unexpected unknown exception in reconstructed sample callback\n";
        std::terminate();
    }
}

} /* namespace sparsdr */
} /* namespace gr */
