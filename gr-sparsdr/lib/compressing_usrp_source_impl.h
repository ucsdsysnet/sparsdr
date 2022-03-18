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

#ifndef INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_IMPL_H
#define INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_IMPL_H

#include <gnuradio/uhd/usrp_source.h>
#include <sparsdr/compressing_usrp_source.h>

namespace gr {
namespace sparsdr {

class compressing_usrp_source_impl : public compressing_usrp_source
{
private:
    /** The inner USRP source */
    gr::uhd::usrp_source::sptr d_usrp;
    /** The configured FFT size */
    std::uint32_t d_fft_size;

public:
    compressing_usrp_source_impl(const ::uhd::device_addr_t& device_addr);
    ~compressing_usrp_source_impl();

    virtual void set_gain(double gain);
    virtual ::uhd::tune_result_t
    set_center_freq(const ::uhd::tune_request_t& tune_request);
    virtual void set_antenna(const std::string& ant);

    virtual void set_compression_enabled(bool enabled) override;
    virtual void set_run_fft(bool enable) override;
    virtual void set_send_average_samples(bool enable) override;
    virtual void set_send_fft_samples(bool enable) override;
    virtual void set_fft_size(std::uint32_t size) override;
    virtual std::uint32_t fft_size() const override;
    virtual void set_shift_amount(std::uint8_t scaling) override;
    virtual void set_bin_threshold(std::uint16_t bin_index,
                                   std::uint32_t threshold) override;
    virtual void set_bin_window_value(std::uint16_t bin_index,
                                      std::uint16_t value) override;
    virtual void set_bin_mask(std::uint16_t bin_index) override;
    virtual void clear_bin_mask(std::uint16_t bin_index) override;
    virtual void set_average_weight(float weight) override;
    virtual void set_average_interval(std::uint32_t interval) override;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_IMPL_H */
