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


#ifndef INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_H
#define INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>
#include <sparsdr/band_spec.h>
#include <sparsdr/compressing_source.h>
#include <uhd/types/device_addr.hpp>
#include <uhd/types/tune_request.hpp>
#include <uhd/types/tune_result.hpp>

namespace gr {
namespace sparsdr {

/*!
 * \brief A combination of a compressing USRP source block and a reconstruct
 * block
 * \ingroup sparsdr
 *
 */
class SPARSDR_API combined_usrp_receiver : virtual public gr::hier_block2,
                                           public compressing_source
{
public:
    typedef boost::shared_ptr<combined_usrp_receiver> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::combined_usrp_receiver.
     *
     * To avoid accidental use of raw pointers, sparsdr::combined_usrp_receiver's
     * constructor is in a private implementation
     * class. sparsdr::combined_usrp_receiver::make is the public interface for
     * creating new instances.
     *
     * \param device_addr The address of the USRP
     * \param format_version The compressed sample format version that
     * corresponds to the FPGA image on the USRP (1 or 2)
     * \param bands A list of bands to reconstruct
     * \param zero_gaps true to insert zero samples in the output(s) for periods
     * when there were no active signals
     */
    static sptr make(const ::uhd::device_addr_t& device_addr,
                     int format_version,
                     const std::vector<band_spec>& bands,
                     const std::string& reconstruct_path = "sparsdr_reconstruct",
                     bool zero_gaps = false);

    // Compressing USRP source delegated functions
    // For documentation, see compressing_usrp_source.h
    virtual void set_gain(double gain) = 0;
    virtual ::uhd::tune_result_t
    set_center_freq(const ::uhd::tune_request_t& tune_request) = 0;
    virtual void set_antenna(const std::string& ant) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_H */
