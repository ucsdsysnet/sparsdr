/* -*- c++ -*- */
/*
 * Copyright 2020 The Regents of the University of California.
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

#include "compressing_plutosdr_source_impl.h"
#include <gnuradio/io_signature.h>

namespace gr {
namespace sparsdr {

compressing_plutosdr_source::sptr compressing_plutosdr_source::make(
    const std::string& uri, unsigned long long frequency, double gain)
{
    return gnuradio::get_initial_sptr(
        new compressing_plutosdr_source_impl(uri, frequency, gain));
}

/*
 * The private constructor
 */
compressing_plutosdr_source_impl::compressing_plutosdr_source_impl(
    const std::string& uri, unsigned long long frequency, double gain)
    : gr::hier_block2("compressing_plutosdr_source",
                      gr::io_signature::make(0, 0, 0),
                      gr::io_signature::make(1, 1, sizeof(std::uint32_t))),
      d_fmcomm(gr::iio::fmcomms2_source::make(uri,
                                              frequency,
                                              /* TODO sample rate */ 40000000,
                                              /* TODO bandwidth */ 40000000,
                                              true,
                                              false,
                                              false,
                                              false,
                                              32768,
                                              true,
                                              true,
                                              true,
                                              "Manual",
                                              30.0,
                                              "Manual",
                                              0.0,
                                              "A_BALANCED"))
{
    connect(d_fmcomm, 0, self(), 0);
}

/*
 * Our virtual destructor.
 */
compressing_plutosdr_source_impl::~compressing_plutosdr_source_impl() {}

} /* namespace sparsdr */
} /* namespace gr */
