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

#ifndef INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_IMPL_H
#define INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_IMPL_H

#include <sparsdr/combined_usrp_receiver.h>
#include <sparsdr/compressing_usrp_source.h>
#include <sparsdr/reconstruct.h>

namespace gr {
namespace sparsdr {

class combined_usrp_receiver_impl : public combined_usrp_receiver
{
private:
    // Enclosed blocks
    compressing_usrp_source::sptr d_usrp;
    reconstruct::sptr d_reconstruct;

public:
    combined_usrp_receiver_impl(const ::uhd::device_addr_t& device_addr,
                                int format_version,
                                const std::vector<band_spec>& bands,
                                const std::string& reconstruct_path,
                                bool zero_gaps);


    // Compressing USRP source delegated functions
    virtual void set_gain(double gain) override;
    virtual ::uhd::tune_result_t
    set_center_freq(const ::uhd::tune_request_t& tune_request) override;
    virtual void set_antenna(const std::string& ant) override;
    virtual void set_compression_enabled(bool enabled) override;
    virtual void set_fft_enabled(bool enabled) override;
    virtual void set_fft_send_enabled(bool enabled) override;
    virtual void set_average_send_enabled(bool enabled) override;
    virtual void start_all() override;
    virtual void stop_all() override;
    virtual void set_fft_size(uint32_t size) override;
    virtual void set_fft_scaling(uint32_t scaling) override;
    virtual void set_threshold(uint16_t index, uint32_t threshold) override;
    virtual void set_mask_enabled(uint16_t index, bool enabled) override;
    virtual void set_average_weight(float weight) override;
    virtual void set_average_packet_interval(uint32_t interval) override;

    ~combined_usrp_receiver_impl();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_IMPL_H */
