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

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "compressing_pluto_source_impl.h"
#include <gnuradio/io_signature.h>

#include <gnuradio/iio/device_source.h>
#include <cmath>
#include <iostream>

namespace gr {
namespace sparsdr {

namespace {
bool is_power_of_two(std::uint32_t value) { return (value & (value - 1)) == 0; }
/**
 * Calculates the base-2 logarithm of an integer, assuming that the
 * integer is a power of two
 */
std::uint32_t int_log2(std::uint32_t value)
{
    // Shift right so that int_log2(1) == 0
    value >>= 1;

    std::uint32_t log = 0;
    while (value != 0) {
        log += 1;
        value >>= 1;
    }
    return log;
}
/**
 * Calculates the base-2 logarithm of an integer, rounded up
 */
std::uint32_t ceiling_log2(std::uint32_t value)
{
    if (is_power_of_two(value)) {
        return int_log2(value);
    } else {
        return 1 + int_log2(value);
    }
}
} // namespace

compressing_pluto_source::sptr compressing_pluto_source::make(const std::string& uri)
{
    return gnuradio::get_initial_sptr(new compressing_pluto_source_impl(uri));
}

/*
 * The private constructor
 */
compressing_pluto_source_impl::compressing_pluto_source_impl(const std::string& uri)
    : gr::hier_block2("compressing_pluto_source",
                      gr::io_signature::make(0, 0, 0),
                      gr::io_signature::make(1, 1, sizeof(short))),
      d_iio_context(nullptr),
      d_sparsdr_device(nullptr)
{
    d_iio_context = iio_create_context_from_uri(uri.c_str());
    if (!d_iio_context) {
        throw std::runtime_error("Can't create IIO context");
    }
    // Find the SparSDR device and configure it
    d_sparsdr_device = iio_context_find_device(d_iio_context, "sparsdr");
    // TODO: Make logging consistent with GNU Radio conventions
    if (!d_sparsdr_device) {
        std::cerr << "SparSDR device not found on the Pluto radio. "
                  << "Check that the sparsdr_iio kernel module has been installed "
                  << "and iiod has been restarted.";
        throw std::runtime_error("No SparSDR device");
    }

    // Create IIO device source block and connect
    // When using the make_from function, the device source will not destroy
    // the IIO context.
    //
    // About the params: Each key is the name of a file under /sys/bus/iio/devices/iio:deviceX .
    const auto source_block =
        gr::iio::device_source::make_from(/* context */ d_iio_context,
                                          /* device */ "cf-ad9361-lpc",
                                          /* channels */ { "voltage0" },
                                          /* PHY */ "ad9361-phy",
                                          /* params */
                                          { "in_voltage_sampling_frequency=61440000",
                                            "in_voltage_rf_bandwidth=56000000",
                                            "in_voltage0_gain_control_mode=manual",
                                            "in_voltage0_hardwaregain=60.0"
                                            "out_altvoltage0_RX_LO_frequency=2412000000" });
    // Increase timeout to 2 seconds
    source_block->set_timeout_ms(2000);
    connect(source_block, 0, self(), 0);
}

void compressing_pluto_source_impl::set_enable_compression(bool enable)
{
    write_bool_attr("enable_compression", enable);
}
void compressing_pluto_source_impl::set_run_fft(bool enable)
{
    write_bool_attr("run_fft", enable);
}
void compressing_pluto_source_impl::set_send_average_samples(bool enable)
{
    write_bool_attr("send_average_samples", enable);
}
void compressing_pluto_source_impl::set_send_fft_samples(bool enable)
{
    write_bool_attr("send_fft_samples", enable);
}
void compressing_pluto_source_impl::start_all()
{
    set_enable_compression(true);
    set_send_fft_samples(true);
    set_send_average_samples(true);
    set_run_fft(true);
}
void compressing_pluto_source_impl::stop_all()
{
    set_run_fft(false);
    set_send_average_samples(false);
    set_send_fft_samples(false);
    set_enable_compression(false);
}
void compressing_pluto_source_impl::set_fft_size(std::uint32_t size)
{
    if (!is_power_of_two(size) || size < 8 || size > 1024) {
        throw std::runtime_error(
            "FFT size must be a power of two between 8 and 1024 inclusive");
    }
    // Register value is the base-2 logarithm of the FFT size
    const unsigned int log_size = int_log2(size);
    write_u32_attr("fft_size", log_size);
}
void compressing_pluto_source_impl::set_bin_threshold(std::uint16_t bin_index,
                                                      std::uint32_t threshold)
{
    // The threshold value is latched when the bin number is written
    write_u32_attr("threshold_value", std::uint32_t(threshold));
    write_u32_attr("threshold_bin_number", bin_index);
}
void compressing_pluto_source_impl::set_bin_window_value(std::uint16_t bin_index,
                                                         std::uint16_t value)
{
    write_u32_attr("window_value",
                   (std::uint32_t(bin_index) << 16) | std::uint32_t(value));
}
void compressing_pluto_source_impl::set_bin_mask(std::uint16_t bin_index)
{
    write_u32_attr("bin_mask", (std::uint32_t(bin_index) << 1) | 0x1);
}
void compressing_pluto_source_impl::clear_bin_mask(std::uint16_t bin_index)
{
    write_u32_attr("bin_mask", std::uint32_t(bin_index));
}
void compressing_pluto_source_impl::set_average_weight(float weight)
{
    if (weight < 0.0 || weight >= 1.0 || std::isnan(weight)) {
        throw std::runtime_error(
            "Average weight must be greater than or equal to 0 and less than 1");
    }
    // Map from [0, 1) to [0, 256)
    const std::uint32_t register_value = uint32_t(weight * 256.0);
    write_u32_attr("average_weight", register_value);
}
void compressing_pluto_source_impl::set_average_interval(std::uint32_t interval)
{
    if (interval < 8 || interval > 2147483648) {
        throw std::runtime_error(
            "Average interval must be between 8 and 2147483648 inclusive");
    }
    // Actual register value is the base-2 logarithm of the interval
    write_u32_attr("average_interval", ceiling_log2(interval));
}

void compressing_pluto_source_impl::write_bool_attr(const char* name, bool value)
{
    const char* value_text = value ? "1" : "0";
    const ssize_t status = iio_device_attr_write(d_sparsdr_device, name, value_text);
    // Expected return value includes one extra byte
    if (status != 2) {
        throw std::runtime_error("Failed to write boolean attribute");
    }
}

void compressing_pluto_source_impl::write_u32_attr(const char* name, std::uint32_t value)
{
    std::stringstream stream;
    stream << value;
    const std::string string_value = stream.str();
    const ssize_t status =
        iio_device_attr_write(d_sparsdr_device, name, string_value.c_str());
    // Expected return value includes one extra byte
    if (status != string_value.length() + 1) {
        throw std::runtime_error("Failed to write u32 attribute");
    }
}

/*
 * Our virtual destructor.
 */
compressing_pluto_source_impl::~compressing_pluto_source_impl()
{
    iio_context_destroy(d_iio_context);
}


} /* namespace sparsdr */
} /* namespace gr */
