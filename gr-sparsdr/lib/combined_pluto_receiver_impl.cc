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

#include "combined_pluto_receiver_impl.h"
#include "combined_common.h"
#include <gnuradio/io_signature.h>

namespace gr {
namespace sparsdr {

namespace {
constexpr float PLUTO_BANDWIDTH = 61.44e6;
constexpr unsigned int PLUTO_FFT_SIZE = 1024;
} // namespace

combined_pluto_receiver::sptr
combined_pluto_receiver::make(const std::string& uri,
                              std::size_t buffer_size,
                              const std::vector<band_spec>& bands,
                              const std::string& reconstruct_path)
{
    return gnuradio::get_initial_sptr(
        new combined_pluto_receiver_impl(uri, buffer_size, bands, reconstruct_path));
}

/*
 * The private constructor
 */
combined_pluto_receiver_impl::combined_pluto_receiver_impl(
    const std::string& uri,
    std::size_t buffer_size,
    const std::vector<band_spec>& bands,
    const std::string& reconstruct_path)
    : gr::hier_block2(
          "combined_pluto_receiver",
          gr::io_signature::make(0, 0, 1),
          gr::io_signature::make(bands.size(), bands.size(), sizeof(std::uint32_t))),
      d_pluto(nullptr),
      d_reconstruct(nullptr)
{
    float center_frequency;
    if (!choose_center_frequency(
            bands, PLUTO_BANDWIDTH, PLUTO_FFT_SIZE, &center_frequency)) {
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

    d_pluto = compressing_pluto_source::make(uri, buffer_size);

    const char* format_version_string;
    switch (d_pluto->format_version()) {
    case 1:
        format_version_string = "N210 v1";
        break;
    case 2:
        format_version_string = "N210 v2";
        break;
    default:
        throw std::runtime_error("Invalid format version, expected 1 or 2");
    }
    d_reconstruct =
        reconstruct::make(relative_bands, reconstruct_path, format_version_string);
    // Connect
    connect(d_pluto, 0, d_reconstruct, 0);
    for (std::size_t i = 0; i < bands.size(); i++) {
        connect(d_reconstruct, i, self(), i);
    }
}

/*
 * Our virtual destructor.
 */
combined_pluto_receiver_impl::~combined_pluto_receiver_impl() {}


} /* namespace sparsdr */
} /* namespace gr */
