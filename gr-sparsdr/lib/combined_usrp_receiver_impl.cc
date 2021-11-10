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

#include "combined_common.h"
#include "combined_usrp_receiver_impl.h"
#include <gnuradio/io_signature.h>
#include <cstdint>
#include <stdexcept>

namespace {

constexpr float N210_BANDWIDTH = 100e6f;
constexpr unsigned int N210_FFT_SIZE = 2048;

} // namespace

namespace gr {
namespace sparsdr {

combined_usrp_receiver::sptr
combined_usrp_receiver::make(const ::uhd::device_addr_t& device_addr,
                             int format_version,
                             const std::vector<band_spec>& bands,
                             const std::string& reconstruct_path)
{
    return gnuradio::get_initial_sptr(new combined_usrp_receiver_impl(
        device_addr, format_version, bands, reconstruct_path));
}

/*
 * The private constructor
 */
combined_usrp_receiver_impl::combined_usrp_receiver_impl(
    const ::uhd::device_addr_t& device_addr,
    int format_version,
    const std::vector<band_spec>& bands,
    const std::string& reconstruct_path)
    : gr::hier_block2(
          "combined_usrp_receiver",
          gr::io_signature::make(0, 0, 1),
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex))),
      d_usrp(nullptr),
      d_reconstruct(nullptr)
{
    float center_frequency;
    if (!choose_center_frequency(
            bands, N210_BANDWIDTH, N210_FFT_SIZE, &center_frequency)) {
        throw std::runtime_error("Can't find an appropriate center frequency");
    }
    std::cout << "Center frequency " << center_frequency << " Hz\n";

    // Convert the bands into bands relative to the center frequency
    std::vector<band_spec> relative_bands;
    relative_bands.reserve(bands.size());
    for (const band_spec& band : bands) {
        relative_bands.push_back(
            band_spec(band.frequency() - center_frequency, band.bins()));
    }
    const char* format_version_string;
    switch (format_version) {
    case 1:
        format_version_string = "N210 v1";
        break;
    case 2:
        format_version_string = "N210 v2";
        break;
    default:
        throw std::runtime_error("Invalid format version, expected 1 or 2");
    }
    // Create blocks
    d_usrp = compressing_usrp_source::make(device_addr);
    d_reconstruct =
        reconstruct::make(relative_bands, reconstruct_path, format_version_string);
    // Connect
    connect(d_usrp, 0, d_reconstruct, 0);
    for (std::size_t i = 0; i < bands.size(); i++) {
        connect(d_reconstruct, i, self(), i);
    }
}

// Compressing USRP source delegated functions
void combined_usrp_receiver_impl::set_gain(double gain) { d_usrp->set_gain(gain); }
::uhd::tune_result_t
combined_usrp_receiver_impl::set_center_freq(const ::uhd::tune_request_t& tune_request)
{
    return d_usrp->set_center_freq(tune_request);
}
void combined_usrp_receiver_impl::set_antenna(const std::string& ant)
{
    d_usrp->set_antenna(ant);
}
void combined_usrp_receiver_impl::set_compression_enabled(bool enabled)
{
    d_usrp->set_compression_enabled(enabled);
}
void combined_usrp_receiver_impl::set_fft_enabled(bool enabled)
{
    d_usrp->set_fft_enabled(enabled);
}
void combined_usrp_receiver_impl::set_fft_send_enabled(bool enabled)
{
    d_usrp->set_fft_send_enabled(enabled);
}
void combined_usrp_receiver_impl::set_average_send_enabled(bool enabled)
{
    d_usrp->set_average_send_enabled(enabled);
}
void combined_usrp_receiver_impl::start_all() { d_usrp->start_all(); }
void combined_usrp_receiver_impl::stop_all() { d_usrp->stop_all(); }
void combined_usrp_receiver_impl::set_fft_size(uint32_t size)
{
    d_usrp->set_fft_size(size);
}
void combined_usrp_receiver_impl::set_fft_scaling(uint32_t scaling)
{
    d_usrp->set_fft_scaling(scaling);
}
void combined_usrp_receiver_impl::set_threshold(uint16_t index, uint32_t threshold)
{
    d_usrp->set_threshold(index, threshold);
}
void combined_usrp_receiver_impl::set_mask_enabled(uint16_t index, bool enabled)
{
    d_usrp->set_mask_enabled(index, enabled);
}
void combined_usrp_receiver_impl::set_average_weight(float weight)
{
    d_usrp->set_average_weight(weight);
}
void combined_usrp_receiver_impl::set_average_packet_interval(uint32_t interval)
{
    d_usrp->set_average_packet_interval(interval);
}

/*
 * Our virtual destructor.
 */
combined_usrp_receiver_impl::~combined_usrp_receiver_impl() {}


} /* namespace sparsdr */
} /* namespace gr */
