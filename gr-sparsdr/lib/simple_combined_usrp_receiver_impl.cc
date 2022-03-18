/* -*- c++ -*- */
/*
 * Copyright 2022 The Regents of the University of California.
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
#include "simple_combined_usrp_receiver_impl.h"
#include <gnuradio/io_signature.h>
#include <sparsdr/combined_usrp_receiver.h>

namespace gr {
namespace sparsdr {

namespace {
constexpr float USRP_SAMPLE_RATE = 100e6;
constexpr float USRP_RECEIVE_BANDWIDTH = 100e6;
constexpr unsigned int USRP_DEFAULT_FFT_SIZE = 2048;

constexpr device_properties USRP_PROPERTIES = { USRP_DEFAULT_FFT_SIZE,
                                                USRP_SAMPLE_RATE,
                                                USRP_RECEIVE_BANDWIDTH };

} // namespace

simple_combined_usrp_receiver::sptr
simple_combined_usrp_receiver::make(const ::uhd::device_addr_t& device_addr,
                                    int format_version,
                                    float center_frequency,
                                    const std::vector<simple_band_spec>& bands,
                                    std::uint32_t threshold,
                                    const std::string& reconstruct_path,
                                    bool zero_gaps)
{
    return gnuradio::get_initial_sptr(
        new simple_combined_usrp_receiver_impl(device_addr,
                                               format_version,
                                               center_frequency,
                                               bands,
                                               threshold,
                                               reconstruct_path,
                                               zero_gaps));
}

/*
 * The private constructor
 */
simple_combined_usrp_receiver_impl::simple_combined_usrp_receiver_impl(
    const ::uhd::device_addr_t& device_addr,
    int format_version,
    float center_frequency,
    const std::vector<simple_band_spec>& bands,
    std::uint32_t threshold,
    const std::string& reconstruct_path,
    bool zero_gaps)
    : gr::hier_block2(
          "simple_combined_usrp_receiver",
          gr::io_signature::make(0, 0, 0),
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex))),
      d_inner_block(nullptr)
{
    // Calculate bins and things
    combined_receiver_setup setup(center_frequency, bands, threshold, USRP_PROPERTIES);

    // Create and configure inner block
    auto inner_block = combined_usrp_receiver::make(device_addr,
                                                    format_version,
                                                    setup.reconstruct_bands,
                                                    reconstruct_path,
                                                    zero_gaps);
    // This configuration doesn't need to be done from the Python code
    inner_block->set_center_freq(center_frequency);
    inner_block->stop_all();
    inner_block->set_fft_size(USRP_DEFAULT_FFT_SIZE);
    inner_block->load_rounded_hann_window(USRP_DEFAULT_FFT_SIZE);
    inner_block->set_bin_spec(setup.generated_bin_spec);
    inner_block->start_all();
    // The gain and shift amount do need to be configured from the Python
    // code (or whatever the client code is)

    // Connect all outputs
    for (std::size_t i = 0; i != bands.size(); i++) {
        connect(inner_block, i, self(), i);
    }
    d_inner_block = inner_block;
}


void simple_combined_usrp_receiver_impl::set_gain(double gain)
{
    d_inner_block->set_gain(gain);
}
void simple_combined_usrp_receiver_impl::set_antenna(const std::string& antenna)
{
    d_inner_block->set_antenna(antenna);
}
void simple_combined_usrp_receiver_impl::set_shift_amount(std::uint8_t scaling)
{
    d_inner_block->set_shift_amount(scaling);
}

/*
 * Our virtual destructor.
 */
simple_combined_usrp_receiver_impl::~simple_combined_usrp_receiver_impl() {}


} /* namespace sparsdr */
} /* namespace gr */
