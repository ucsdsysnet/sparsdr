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

    // compressing_source function implementations
    virtual void set_compression_enabled(bool enable) override;
    virtual void set_run_fft(bool enable) override;
    virtual void set_send_fft_samples(bool enable) override;
    virtual void set_send_average_samples(bool enable) override;
    virtual void set_fft_size(std::uint32_t size) override;
    virtual std::uint32_t fft_size() const override;
    virtual void set_shift_amount(std::uint8_t scaling) override;
    virtual void set_bin_threshold(std::uint16_t index, std::uint32_t threshold) override;
    virtual void set_bin_window_value(std::uint16_t bin_index,
                                      std::uint16_t value) override;
    virtual void set_bin_mask(std::uint16_t bin_index) override;
    virtual void clear_bin_mask(std::uint16_t bin_index) override;
    virtual void set_average_weight(float weight) override;
    virtual void set_average_interval(std::uint32_t interval) override;

    ~combined_usrp_receiver_impl();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMBINED_USRP_RECEIVER_IMPL_H */
