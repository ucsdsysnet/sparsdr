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

#ifndef INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_IMPL_H
#define INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_IMPL_H

#include <iio.h>
#include <sparsdr/compressing_pluto_source.h>

namespace gr {
namespace sparsdr {

class compressing_pluto_source_impl : public compressing_pluto_source
{
private:
    /**
     * IIO contex used to connect to the radio
     *
     * This is associated with d_sparsdr_device and used by the device source
     * block.
     */
    iio_context* d_iio_context;
    /**
     * A connection to the SparSDR IIO device
     *
     * This is used to change compression settings.
     */
    iio_device* d_sparsdr_device;

    /** IIO device used for tuning and gain */
    iio_device* d_ad9361_phy;

    /**
     * Writes a boolean attribute of the SparSDR device
     */
    void write_bool_attr(const char* name, bool value);
    /**
     * Writes a 32-bit unsigned integer attribute of the SparSDR device
     */
    void write_u32_attr(const char* name, std::uint32_t value);

public:
    compressing_pluto_source_impl(const std::string& uri);

    virtual void set_frequency(unsigned long long frequency);
    virtual void set_gain(double gain);

    virtual void set_enable_compression(bool enable);
    virtual void set_run_fft(bool enable);
    virtual void set_send_average_samples(bool enable);
    virtual void set_send_fft_samples(bool enable);
    virtual void start_all();
    virtual void stop_all();
    virtual void set_fft_size(std::uint32_t size);
    virtual void set_bin_threshold(std::uint16_t bin_index, std::uint32_t threshold);
    virtual void set_bin_window_value(std::uint16_t bin_index, std::uint16_t value);
    virtual void set_bin_mask(std::uint16_t bin_index);
    virtual void clear_bin_mask(std::uint16_t bin_index);
    virtual void set_average_weight(float weight);
    virtual void set_average_interval(std::uint32_t interval);

    ~compressing_pluto_source_impl();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_IMPL_H */
