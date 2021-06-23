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

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <stdexcept>

#include "compressing_usrp_source_impl.h"
#include <gnuradio/io_signature.h>
#include <sparsdr/detail/registers.h>

namespace gr {
namespace sparsdr {

namespace {
/**
 * Returns the number of leading zeros in the binary representation of
 * a number
 */
uint32_t leading_zeros(uint32_t value)
{
    uint32_t zeros = 0;
    while ((value >> 31) == 0 && zeros < 32) {
        value <<= 1;
        zeros += 1;
    }
    return zeros;
}
} // namespace

namespace registers = gr::sparsdr::detail::registers;

compressing_usrp_source::sptr
compressing_usrp_source::make(const ::uhd::device_addr_t& device_addr)
{
    return gnuradio::get_initial_sptr(new compressing_usrp_source_impl(device_addr));
}

/*
 * The private constructor
 */
compressing_usrp_source_impl::compressing_usrp_source_impl(
    const ::uhd::device_addr_t& device_addr)
    : gr::hier_block2("compressing_usrp_source",
                      gr::io_signature::make(0, 0, 0),
                      gr::io_signature::make(1, 1, sizeof(std::uint32_t))),
      d_usrp(gr::uhd::usrp_source::make(
          device_addr,
          // Always use sc16 to prevent interpreting the samples as numbers
          ::uhd::stream_args_t("sc16", "sc16")))
{
    // Connect the all-important output
    connect(d_usrp, 0, self(), 0);
}

/*
 * Our virtual destructor.
 */
compressing_usrp_source_impl::~compressing_usrp_source_impl() {}


// USRP settings

void compressing_usrp_source_impl::set_gain(double gain) { d_usrp->set_gain(gain); }
::uhd::tune_result_t
compressing_usrp_source_impl::set_center_freq(const ::uhd::tune_request_t tune_request)
{
    return d_usrp->set_center_freq(tune_request);
}
void compressing_usrp_source_impl::set_antenna(const std::string& ant)
{
    d_usrp->set_antenna(ant);
}


// SparSDR-specific settings

void compressing_usrp_source_impl::set_compression_enabled(bool enabled)
{
    d_usrp->set_user_register(registers::ENABLE_COMPRESSION, enabled);
}

void compressing_usrp_source_impl::set_fft_enabled(bool enabled)
{
    d_usrp->set_user_register(registers::RUN_FFT, enabled);
}

void compressing_usrp_source_impl::set_fft_send_enabled(bool enabled)
{
    d_usrp->set_user_register(registers::FFT_SEND, enabled);
}

void compressing_usrp_source_impl::set_average_send_enabled(bool enabled)
{
    d_usrp->set_user_register(registers::AVG_SEND, enabled);
}

void compressing_usrp_source_impl::start_all()
{
    set_fft_send_enabled(true);
    set_average_send_enabled(true);
    set_fft_enabled(true);
}

void compressing_usrp_source_impl::stop_all()
{
    set_fft_enabled(false);
    set_average_send_enabled(false);
    set_fft_send_enabled(false);
}

void compressing_usrp_source_impl::set_fft_size(uint32_t size)
{
    d_usrp->set_user_register(registers::FFT_SIZE, size);
}

void compressing_usrp_source_impl::set_fft_scaling(uint32_t scaling)
{
    d_usrp->set_user_register(registers::SCALING, scaling);
}

void compressing_usrp_source_impl::set_threshold(uint16_t index, uint32_t threshold)
{
    // Register format:
    // Bits 31:21 : index (11 bits)
    // Bits 20:0 : threshold shifted right by 11 bits (21 bits)

    // Check that index fits within 11 bits
    if (index > 0x7ffu) {
        throw std::out_of_range("index must fit within 11 bits");
    }

    const uint32_t command = (index << 21) | (threshold >> 11);
    d_usrp->set_user_register(registers::THRESHOLD, command);
}

void compressing_usrp_source_impl::set_mask_enabled(uint16_t index, bool enabled)
{
    // Register format:
    // Bits 31:1 : index (31 bits)
    // Bit 0 : set mask (1) / clear mask (0)

    // Check that index fits within 31 bits
    if (index > 0x7fffffffu) {
        throw std::out_of_range("index must fit within 31 bits");
    }
    const uint32_t command = (index << 1) | enabled;
    d_usrp->set_user_register(registers::MASK, command);
}

void compressing_usrp_source_impl::set_average_weight(float weight)
{
    if (weight < 0.0 || weight > 1.0) {
        throw std::out_of_range("weight must be in the range [0, 1]");
    }
    // Map to 0...255
    const uint8_t mapped = static_cast<uint8_t>(weight * 255.0);
    d_usrp->set_user_register(registers::AVG_WEIGHT, mapped);
}

void compressing_usrp_source_impl::set_average_packet_interval(uint32_t interval)
{
    if (interval == 0) {
        throw std::out_of_range("interval must not be 0");
    }
    // Register format: ceiling of the base-2 logarithm of the interval
    const uint32_t ceiling_log_interval = 31 - leading_zeros(interval);
    d_usrp->set_user_register(registers::AVG_INTERVAL, ceiling_log_interval);
}

} /* namespace sparsdr */
} /* namespace gr */
