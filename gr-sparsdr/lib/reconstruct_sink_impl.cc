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

#include "reconstruct_sink_impl.h"
#include <gnuradio/io_signature.h>

namespace gr {
namespace sparsdr {

namespace {

/**
 * Size of a GNU Radio input sample in bytes
 * (the size that the compressed sample parser uses may be different)
 */
constexpr std::size_t GR_IN_SAMPLE_BYTES = sizeof(std::uint32_t);

} // namespace

reconstruct_sink::sptr
reconstruct_sink::make(::sparsdr::sparsdr_reconstruct_context* context,
                       int parser_sample_bytes)
{
    return gnuradio::get_initial_sptr(
        new reconstruct_sink_impl(context, parser_sample_bytes));
}


/*
 * The private constructor
 */
reconstruct_sink_impl::reconstruct_sink_impl(
    ::sparsdr::sparsdr_reconstruct_context* context, int parser_sample_bytes)
    : gr::sync_block("reconstruct_sink",
                     gr::io_signature::make(1, 1, GR_IN_SAMPLE_BYTES),
                     gr::io_signature::make(0, 0, 0)),
      d_context(context),
      d_parser_sample_bytes(parser_sample_bytes)
{
}

int reconstruct_sink_impl::work(int noutput_items,
                                gr_vector_const_void_star& input_items,
                                gr_vector_void_star&)
{
    const std::size_t num_input_bytes = noutput_items * GR_IN_SAMPLE_BYTES;
    const std::size_t input_compressed_samples = num_input_bytes / d_parser_sample_bytes;
    const std::uint8_t* input_bytes =
        reinterpret_cast<const std::uint8_t*>(input_items[0]);
    for (std::size_t i = 0; i < input_compressed_samples; i++) {
        const std::uint8_t* sample = input_bytes + (i * d_parser_sample_bytes);
        const int status = sparsdr_reconstruct_handle_samples(
            d_context, reinterpret_cast<const void*>(sample), d_parser_sample_bytes);
        if (status != 0) {
            std::cerr << "sparsdr_reconstruct_handle_samples returned " << status << '\n';
            return WORK_DONE;
        }
    }
    // Consumed all items
    return noutput_items;
}

/*
 * Our virtual destructor.
 */
reconstruct_sink_impl::~reconstruct_sink_impl()
{
    // Free the reconstruction resources
    ::sparsdr::sparsdr_reconstruct_free(d_context);
    d_context = nullptr;
}

} /* namespace sparsdr */
} /* namespace gr */
