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
#include "simple_combined_pluto_receiver_impl.h"
#include <gnuradio/io_signature.h>
#include <sparsdr/combined_pluto_receiver.h>

namespace gr {
namespace sparsdr {

namespace {
constexpr float PLUTO_SAMPLE_RATE = 61.44e6;
constexpr float PLUTO_RECEIVE_BANDWIDTH = 56e6;
constexpr unsigned int PLUTO_DEFAULT_FFT_SIZE = 1024;

constexpr device_properties PLUTO_PROPERTIES = { PLUTO_DEFAULT_FFT_SIZE,
                                                 PLUTO_SAMPLE_RATE,
                                                 PLUTO_RECEIVE_BANDWIDTH };

} // namespace

simple_combined_pluto_receiver::sptr
simple_combined_pluto_receiver::make(const std::string& uri,
                                     std::size_t buffer_size,
                                     float center_frequency,
                                     const std::vector<simple_band_spec>& bands,
                                     std::uint32_t threshold,
                                     const std::string& reconstruct_path,
                                     bool zero_gaps,
                                     bool skip_bin_config)
{
    return gnuradio::get_initial_sptr(
        new simple_combined_pluto_receiver_impl(uri,
                                                buffer_size,
                                                center_frequency,
                                                bands,
                                                threshold,
                                                reconstruct_path,
                                                zero_gaps,
                                                skip_bin_config));
}

/*
 * The private constructor
 */
simple_combined_pluto_receiver_impl::simple_combined_pluto_receiver_impl(
    const std::string& uri,
    std::size_t buffer_size,
    float center_frequency,
    const std::vector<simple_band_spec>& bands,
    std::uint32_t threshold,
    const std::string& reconstruct_path,
    bool zero_gaps,
    bool skip_bin_config)
    : gr::hier_block2(
          "simple_combined_pluto_receiver",
          gr::io_signature::make(0, 0, 0),
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex))),
      d_inner_block(nullptr)
{
    // Calculate bins and things
    combined_receiver_setup setup(center_frequency, bands, threshold, PLUTO_PROPERTIES);

    // Create and configure inner block
    auto inner_block = combined_pluto_receiver::make(uri,
                                                     buffer_size,
                                                     PLUTO_DEFAULT_FFT_SIZE,
                                                     center_frequency,
                                                     setup.reconstruct_bands,
                                                     reconstruct_path,
                                                     zero_gaps);
    // This configuration doesn't need to be done from the Python code
    inner_block->set_frequency(static_cast<unsigned long long>(center_frequency));
    inner_block->stop_all();
    if (!skip_bin_config) {
        inner_block->set_fft_size(PLUTO_DEFAULT_FFT_SIZE);
        inner_block->load_rounded_hann_window(PLUTO_DEFAULT_FFT_SIZE);
        inner_block->set_bin_spec(setup.generated_bin_spec);
    }
    inner_block->start_all();
    // The gain and shift amount do need to be configured from the Python
    // code (or whatever the client code is)

    // Connect all outputs
    for (std::size_t i = 0; i != bands.size(); i++) {
        connect(inner_block, i, self(), i);
    }
    d_inner_block = inner_block;
}


void simple_combined_pluto_receiver_impl::set_gain(double gain)
{
    d_inner_block->set_gain(gain);
}
void simple_combined_pluto_receiver_impl::set_shift_amount(std::uint8_t scaling)
{
    d_inner_block->set_shift_amount(scaling);
}

/*
 * Our virtual destructor.
 */
simple_combined_pluto_receiver_impl::~simple_combined_pluto_receiver_impl() {}


} /* namespace sparsdr */
} /* namespace gr */
