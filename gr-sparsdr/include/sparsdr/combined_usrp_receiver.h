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
class SPARSDR_API combined_usrp_receiver : virtual public gr::hier_block2
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
     */
    static sptr make(const ::uhd::device_addr_t& device_addr,
                     int format_version,
                     const std::vector<band_spec>& bands,
                     const std::string& reconstruct_path = "sparsdr_reconstruct");

    // Compressing USRP source delegated functions
    // For documentation, see compressing_usrp_source.h
    virtual void set_gain(double gain) = 0;
    virtual ::uhd::tune_result_t
    set_center_freq(const ::uhd::tune_request_t& tune_request) = 0;
    virtual void set_antenna(const std::string& ant) = 0;
    virtual void set_compression_enabled(bool enabled) = 0;
    virtual void set_fft_enabled(bool enabled) = 0;
    virtual void set_fft_send_enabled(bool enabled) = 0;
    virtual void set_average_send_enabled(bool enabled) = 0;
    virtual void start_all() = 0;
    virtual void stop_all() = 0;
    // set_fft_size is left out because this block and the reconstruct block
    // currently do not support any non-standard FFT sizes.
    virtual void set_fft_scaling(uint32_t scaling) = 0;
    virtual void set_threshold(uint16_t index, uint32_t threshold) = 0;
    virtual void set_mask_enabled(uint16_t index, bool enabled) = 0;
    virtual void set_average_weight(float weight) = 0;
    virtual void set_average_packet_interval(uint32_t interval) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_H */
