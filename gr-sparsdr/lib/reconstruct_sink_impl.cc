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
reconstruct_sink::make(::sparsdr::sparsdr_reconstruct_context* context)
{
    return gnuradio::get_initial_sptr(new reconstruct_sink_impl(context));
}


/*
 * The private constructor
 */
    reconstruct_sink_impl::reconstruct_sink_impl(::sparsdr::sparsdr_reconstruct_context* context)
      : gr::sync_block("reconstruct_sink",
              gr::io_signature::make(1, 1, GR_IN_SAMPLE_BYTES),
              gr::io_signature::make(0, 0, 0))
    {
    }

    /*
     * Our virtual destructor.
     */
    reconstruct_sink_impl::~reconstruct_sink_impl() {}

    int reconstruct_sink_impl::work(int noutput_items,
                                    gr_vector_const_void_star& input_items,
                                    gr_vector_void_star& output_items)
    {
        // TODO
        return noutput_items;
    }

    } /* namespace sparsdr */
    } /* namespace gr */
