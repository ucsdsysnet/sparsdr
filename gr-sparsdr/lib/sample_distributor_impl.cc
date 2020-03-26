/* -*- c++ -*- */
/*
 * Copyright 2019, 2020 The Regents of the University of California.
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

#include <algorithm>
#include <iostream>

#include "sample_distributor_impl.h"
#include <gnuradio/io_signature.h>

namespace gr {
namespace sparsdr {

sample_distributor::sptr
sample_distributor::make(int inputs, const std::vector<std::string>& pipe_paths)
{
    return gnuradio::get_initial_sptr(new sample_distributor_impl(inputs, pipe_paths));
}

/*
 * The private constructor
 */
sample_distributor_impl::sample_distributor_impl(
    int item_size, const std::vector<std::string>& pipe_paths)
    : gr::block("sample_distributor",
                // No inputs (samples are read from named pipes)
                gr::io_signature::make(0, 0, 0),
                // Any number of outputs
                gr::io_signature::make(0, gr::io_signature::IO_INFINITE, item_size)),
      d_pipe_reader(pipe_paths),
      d_item_size(item_size),
      d_inputs(),
      d_decoder_surplus(0)
{
}

/*
 * Our virtual destructor.
 */
sample_distributor_impl::~sample_distributor_impl() {}

void sample_distributor_impl::forecast(int noutput_items,
                                       gr_vector_int& ninput_items_required)
{
    /* <+forecast+> e.g. ninput_items_required[0] = noutput_items */

    // Initialize: No input items required. Inputs are read from the named
    // pipes instead.
    std::fill(ninput_items_required.begin(), ninput_items_required.end(), 0);
}

int sample_distributor_impl::general_work(int noutput_items,
                                          gr_vector_int& ninput_items,
                                          gr_vector_const_void_star& input_items,
                                          gr_vector_void_star& output_items)
{
    // noutput_items: Maximum number of items to write to each output
    // ninput_items: Number of items available to read from the various
    //     inputs

    // const <+ITYPE+> *in = (const <+ITYPE+> *) input_items[0];
    // <+OTYPE+> *out = (<+OTYPE+> *) output_items[0];

    if (d_pipe_reader.all_pipes_closed()) {
        // No more samples will appear
        return WORK_DONE;
    }

    // Block until a sample is available from some named pipe
    d_pipe_reader.wait_for_samples();
    // This unblock may have been caused by samples to read, a pipe closing,
    // or maybe even a signal or some other strange thing. The following code
    // needs to work correctly even if no samples are available.

    // Keep track of the decoder surplus in this call to general_work()
    int local_decoder_surplus = 0;

    // Ensure a used value is available for each output (initialize to false)
    d_outputs_used.resize(output_items.size(), false);

    // Check all inputs
    for (std::size_t in_index = 0; in_index != d_pipe_reader.size(); in_index++) {
        input_info& in_info = d_inputs.at(in_index);

        if (in_info.d_output != input_info::NO_OUTPUT) {
            const int out_index = in_info.d_output;
            // Try to copy samples to the output
            const ssize_t bytes_copied = d_pipe_reader.read_samples(
                in_index, output_items[out_index], noutput_items);
            if (bytes_copied == -1) {
                // No samples available right now
                // TODO: Possibly mark the output as unused
            } else if (bytes_copied == 0) {
                // This pipe is closed. No more samples will appear.
                in_info.d_output = input_info::NO_OUTPUT;
                // Mark this output as unused
                d_outputs_used[out_index] = false;
                in_info.d_output = input_info::NO_OUTPUT;
            } else {
                // OK
                if (bytes_copied % d_item_size != 0) {
                    std::cerr << "Warning: Read " << bytes_copied
                              << " bytes from a named pipe, which is not a multiple of "
                                 "the item size "
                              << d_item_size << ". Results may be incorrect.\n";
                }
                produce(out_index, bytes_copied / d_item_size);
            }
        } else if (!d_pipe_reader.pipe_closed(in_index)) {
            // No output is assigned, but this input may have samples available.

            // Try to find an unused output
            const std::vector<bool>::iterator unused_entry =
                std::find(d_outputs_used.begin(), d_outputs_used.end(), false);
            if (unused_entry == d_outputs_used.end()) {
                std::cerr << "Warning: No output available for possible input "
                          << in_index << '\n';
            } else {
                // Mark this output as used and assign the input
                const std::size_t out_index =
                    std::distance(d_outputs_used.begin(), unused_entry);
                *unused_entry = true;
                in_info.d_output = out_index;

                // Copy samples
                const ssize_t bytes_copied = d_pipe_reader.read_samples(
                    in_index, output_items[out_index], noutput_items);
                if (bytes_copied == -1) {
                    // No samples available right now
                    // TODO: Possibly mark the output as unused
                } else if (bytes_copied == 0) {
                    // This pipe is closed. No more samples will appear.
                    in_info.d_output = input_info::NO_OUTPUT;
                    // Mark this output as unused
                    d_outputs_used[out_index] = false;
                    in_info.d_output = input_info::NO_OUTPUT;
                } else {
                    // OK
                    if (bytes_copied % d_item_size != 0) {
                        std::cerr
                            << "Warning: Read " << bytes_copied
                            << " bytes from a named pipe, which is not a multiple of "
                               "the item size "
                            << d_item_size << ". Results may be incorrect.\n";
                    }
                    produce(out_index, bytes_copied / d_item_size);
                }
            }
        }
    }

    // Update the atomic decoder surplus value
    // This is equal to the number of unused outputs
    d_decoder_surplus = std::count(d_outputs_used.begin(), d_outputs_used.end(), false);

    // This special value allows different numbers of output samples for
    // different outputs, specified by calling produce()
    return WORK_CALLED_PRODUCE;
}

void sample_distributor_impl::add_source_tag(int in_index, int out_index)
{
    gr::tag_t tag;
    tag.offset = nitems_written(out_index);
    tag.key = pmt::intern("source");
    tag.value = pmt::from_long(in_index);
    tag.srcid = pmt::intern("sample_distributor");
    add_item_tag(out_index, tag);
}

int sample_distributor_impl::decoder_surplus() const
{
    // Atomic read
    return d_decoder_surplus;
}

} /* namespace sparsdr */
} /* namespace gr */
