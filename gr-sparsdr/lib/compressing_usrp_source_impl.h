/* -*- c++ -*- */
/*
 * Copyright 2019 The Regents of the University of California.
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

#include <sparsdr/compressing_usrp_source.h>

namespace gr {
namespace sparsdr {

class compressing_usrp_source_impl : public compressing_usrp_source
{
private:
    // The inner USRP source
    gr::uhd::usrp_source::sptr d_usrp;

public:
    compressing_usrp_source_impl(const ::uhd::device_addr_t& device_addr);
    ~compressing_usrp_source_impl();

    virtual void set_gain(double gain);
    virtual ::uhd::tune_result_t
    set_center_freq(const ::uhd::tune_request_t tune_request);
    virtual void set_antenna(const std::string& ant);

    virtual void set_compression_enabled(bool enabled);
    virtual void set_fft_enabled(bool enabled);
    virtual void set_fft_send_enabled(bool enabled);
    virtual void set_average_send_enabled(bool enabled);
    virtual void start_all();
    virtual void stop_all();
    virtual void set_fft_size(uint32_t size);
    virtual void set_fft_scaling(uint32_t scaling);
    virtual void set_threshold(uint16_t index, uint32_t threshold);
    virtual void set_mask_enabled(uint16_t index, bool enabled);
    virtual void set_average_weight(float weight);
    virtual void set_average_packet_interval(uint32_t interval);
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_IMPL_H */
