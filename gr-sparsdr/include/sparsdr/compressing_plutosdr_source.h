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


#ifndef INCLUDED_SPARSDR_COMPRESSING_PLUTOSDR_SOURCE_H
#define INCLUDED_SPARSDR_COMPRESSING_PLUTOSDR_SOURCE_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>

namespace gr {
namespace sparsdr {

/*!
 * \brief Receives compressed signals from a PlutoSDR device
 * \ingroup sparsdr
 *
 */
class SPARSDR_API compressing_plutosdr_source : virtual public gr::hier_block2
{
public:
    typedef boost::shared_ptr<compressing_plutosdr_source> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of
     * sparsdr::compressing_plutosdr_source.
     *
     * To avoid accidental use of raw pointers, sparsdr::compressing_plutosdr_source's
     * constructor is in a private implementation
     * class. sparsdr::compressing_plutosdr_source::make is the public interface for
     * creating new instances.
     */
    static sptr make(const std::string& uri, unsigned long long frequency, double gain);
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_PLUTOSDR_SOURCE_H */
