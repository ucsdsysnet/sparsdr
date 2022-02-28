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

#include "simple_combined_pluto_receiver_impl.h"
#include <gnuradio/io_signature.h>
#include <sparsdr/combined_pluto_receiver.h>

namespace gr {
namespace sparsdr {

simple_combined_pluto_receiver::sptr
simple_combined_pluto_receiver::make(const std::string& uri,
                                     std::size_t buffer_size,
                                     const std::vector<simple_band_spec>& bands,
                                     const std::string& reconstruct_path,
                                     bool zero_gaps)
{
    return gnuradio::get_initial_sptr(new simple_combined_pluto_receiver_impl(
        uri, buffer_size, bands, reconstruct_path, zero_gaps));
}

/*
 * The private constructor
 */
simple_combined_pluto_receiver_impl::simple_combined_pluto_receiver_impl(
    const std::string& uri,
    std::size_t buffer_size,
    const std::vector<simple_band_spec>& bands,
    const std::string& reconstruct_path,
    bool zero_gaps)
    : gr::hier_block2(
          "simple_combined_pluto_receiver",
          gr::io_signature::make(0, 0, 0),
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex)))
{
    // TODO
    std::vector<band_spec> converted_bands;
    // TODO
    auto inner_block = combined_pluto_receiver::make(
        uri, buffer_size, converted_bands, reconstruct_path, zero_gaps);
    // Connect all outputs
    for (std::size_t i = 0; i != bands.size(); i++) {
        connect(inner_block, i, self(), i);
    }
}

/*
 * Our virtual destructor.
 */
simple_combined_pluto_receiver_impl::~simple_combined_pluto_receiver_impl() {}


} /* namespace sparsdr */
} /* namespace gr */
