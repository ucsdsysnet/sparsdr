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

#include "fft_bin_calculator.h"
#include "simple_combined_pluto_receiver_impl.h"
#include <gnuradio/io_signature.h>
#include <sparsdr/combined_pluto_receiver.h>
#include <iostream>
#include <sstream>

namespace gr {
namespace sparsdr {

namespace {
constexpr float PLUTO_SAMPLE_RATE = 61.44e6;
constexpr float PLUTO_RECEIVE_BANDWIDTH = 56e6;
constexpr unsigned int PLUTO_DEFAULT_FFT_SIZE = 1024;
} // namespace

simple_combined_pluto_receiver::sptr
simple_combined_pluto_receiver::make(const std::string& uri,
                                     std::size_t buffer_size,
                                     float center_frequency,
                                     const std::vector<simple_band_spec>& bands,
                                     std::uint32_t threshold,
                                     const std::string& reconstruct_path,
                                     bool zero_gaps)
{
    return gnuradio::get_initial_sptr(
        new simple_combined_pluto_receiver_impl(uri,
                                                buffer_size,
                                                center_frequency,
                                                bands,
                                                threshold,
                                                reconstruct_path,
                                                zero_gaps));
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
    bool zero_gaps)
    : gr::hier_block2(
          "simple_combined_pluto_receiver",
          gr::io_signature::make(0, 0, 0),
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex))),
      d_inner_block(nullptr)
{
    // Determine the bins for each requested band
    std::vector<band_spec> reconstruct_bands;
    std::stringstream generated_bin_spec;
    for (std::size_t i = 0; i != bands.size(); i++) {
        const simple_band_spec& requested_band = bands.at(i);
        exact_ranges band_calculated_ranges;
        const int calc_status = bins_calc_hertz(center_frequency,
                                                PLUTO_SAMPLE_RATE,
                                                requested_band.frequency(),
                                                requested_band.bandwidth(),
                                                PLUTO_RECEIVE_BANDWIDTH,
                                                PLUTO_DEFAULT_FFT_SIZE,
                                                &band_calculated_ranges);

        unsigned int total_bins;

        std::cout << "Band " << i << " (center " << requested_band.frequency()
                  << " Hz, bandwidth " << requested_band.bandwidth() << " Hz): ";
        if (calc_status == 0) {
            std::cout << "Can't determine bins to unmask\n";
            throw std::runtime_error("Can't determine bins to unmask");
        } else if (calc_status == 1) {
            std::cout << "Unmasking bins " << band_calculated_ranges.l_bin1
                      << " (inclusive) to " << band_calculated_ranges.r_bin1
                      << " (exclusive)\n";
            // Append bins to the band specification, with a trailing comma
            generated_bin_spec << band_calculated_ranges.l_bin1 << ".."
                               << band_calculated_ranges.r_bin1 << ":" << threshold
                               << ",";
            total_bins = band_calculated_ranges.r_bin1 - band_calculated_ranges.l_bin1;
        } else {
            std::cout << "Unmasking bins " << band_calculated_ranges.l_bin1
                      << " (inclusive) to " << band_calculated_ranges.r_bin1
                      << " (exclusive) and bins " << band_calculated_ranges.l_bin2
                      << " (inclusive) to " << band_calculated_ranges.r_bin2
                      << " (exclusive)\n";

            // Append two ranges of bins to the band specification, with a trailing comma
            generated_bin_spec << band_calculated_ranges.l_bin1 << ".."
                               << band_calculated_ranges.r_bin1 << ":" << threshold << ","
                               << band_calculated_ranges.l_bin2 << ".."
                               << band_calculated_ranges.r_bin2 << ":" << threshold
                               << ",";

            total_bins = (band_calculated_ranges.r_bin1 - band_calculated_ranges.l_bin1) +
                         (band_calculated_ranges.r_bin2 - band_calculated_ranges.l_bin2);
        }

        // Assemble a bin specification for the inner block
        // This uses absolute frequencies.
        reconstruct_bands.push_back(band_spec(requested_band.frequency(), total_bins));
    }
    // If the bin specification is not empty, remove the trailing comman
    std::string generated_bin_spec_string = generated_bin_spec.str();
    if (!generated_bin_spec_string.empty()) {
        generated_bin_spec_string.pop_back();
    }
    std::cerr << "Generated bin specification: " << generated_bin_spec_string << '\n';

    // Create and configure inner block
    auto inner_block = combined_pluto_receiver::make(uri,
                                                     buffer_size,
                                                     PLUTO_DEFAULT_FFT_SIZE,
                                                     center_frequency,
                                                     reconstruct_bands,
                                                     reconstruct_path,
                                                     zero_gaps);
    // This configuration doesn't need to be done from the Python code
    inner_block->set_frequency(static_cast<unsigned long long>(center_frequency));
    inner_block->stop_all();
    inner_block->set_fft_size(PLUTO_DEFAULT_FFT_SIZE);
    inner_block->load_rounded_hann_window(PLUTO_DEFAULT_FFT_SIZE);
    inner_block->set_bin_spec(generated_bin_spec_string);
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
