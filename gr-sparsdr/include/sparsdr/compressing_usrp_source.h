/* -*- c++ -*- */
/*
 * Copyright 2019-2022 The Regents of the University of California.
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


#ifndef INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_H
#define INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>
#include <sparsdr/compressing_source.h>
#include <uhd/types/device_addr.hpp>
#include <uhd/types/tune_request.hpp>
#include <uhd/types/tune_result.hpp>

namespace gr {
namespace sparsdr {

/*!
 * \brief A block that wraps a UHD USRP source and allows SparSDR
 * compression settings to be changed
 * \ingroup sparsdr
 *
 */
class SPARSDR_API compressing_usrp_source : virtual public gr::hier_block2,
                                            public compressing_source
{
public:
    typedef boost::shared_ptr<compressing_usrp_source> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::compressing_usrp_source.
     *
     * To avoid accidental use of raw pointers, sparsdr::compressing_usrp_source's
     * constructor is in a private implementation
     * class. sparsdr::compressing_usrp_source::make is the public interface for
     * creating new instances.
     */
    static sptr make(const ::uhd::device_addr_t& device_addr);

    // Begin general USRP settings

    /*!
     * Set the gain.
     *
     * \param gain the gain in dB
     */
    virtual void set_gain(double gain) = 0;

    /*!
     * Tune to the desired center frequency.
     *
     * \param tune_request the tune request instructions
     * \return a tune result with the actual frequencies
     */
    virtual ::uhd::tune_result_t
    set_center_freq(const ::uhd::tune_request_t& tune_request) = 0;

    /*!
     * Set the antenna to use.
     *
     * \param ant the antenna string
     */
    virtual void set_antenna(const std::string& ant) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_H */
