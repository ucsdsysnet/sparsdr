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

#include "swap_16_impl.h"
#include <gnuradio/io_signature.h>
#include <cstdint>

namespace {

inline std::uint32_t swap_chunks(std::uint32_t value)
{
    // On x86 this compiles to "rol <some register> 16"
    return (value >> 16) | ((value & 0xffff) << 16);
}

} // namespace

namespace gr {
namespace sparsdr {

swap_16::sptr swap_16::make() { return gnuradio::get_initial_sptr(new swap_16_impl()); }

/*
 * The private constructor
 */
swap_16_impl::swap_16_impl()
    : gr::sync_block("swap_16",
                     gr::io_signature::make(1, 1, sizeof(std::uint32_t)),
                     gr::io_signature::make(1, 1, sizeof(std::uint32_t)))
{
}

/*
 * Our virtual destructor.
 */
swap_16_impl::~swap_16_impl() {}

int swap_16_impl::work(int noutput_items,
                       gr_vector_const_void_star& input_items,
                       gr_vector_void_star& output_items)
{
    const std::uint32_t* in = reinterpret_cast<const std::uint32_t*>(input_items[0]);
    std::uint32_t* out = reinterpret_cast<std::uint32_t*>(output_items[0]);

    for (int i = 0; i < noutput_items; i++) {
        out[i] = swap_chunks(in[i]);
    }

    // Tell runtime system how many output items we produced.
    return noutput_items;
}

} /* namespace sparsdr */
} /* namespace gr */
