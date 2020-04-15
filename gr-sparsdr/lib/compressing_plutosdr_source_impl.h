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

#ifndef INCLUDED_SPARSDR_COMPRESSING_PLUTOSDR_SOURCE_IMPL_H
#define INCLUDED_SPARSDR_COMPRESSING_PLUTOSDR_SOURCE_IMPL_H

#include <gnuradio/iio/fmcomms2_source.h>
#include <sparsdr/compressing_plutosdr_source.h>

namespace gr {
namespace sparsdr {

class compressing_plutosdr_source_impl : public compressing_plutosdr_source
{
private:
    gr::iio::fmcomms2_source::sptr d_fmcomm;

public:
    compressing_plutosdr_source_impl(const std::string& uri,
                                     unsigned long long frequency,
                                     double gain);
    ~compressing_plutosdr_source_impl();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_PLUTOSDR_SOURCE_IMPL_H */
