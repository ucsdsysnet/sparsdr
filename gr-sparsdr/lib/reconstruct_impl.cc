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

#include "reconstruct_impl.h"
#include <gnuradio/io_signature.h>
#include <sparsdr_reconstruct.hpp>

namespace gr {
namespace sparsdr {

namespace {

/**
 * The reconstruction library calls this function (potentially from many
 * different threads) when it has produced samples
 */
void handle_reconstructed_samples(void* context,
                                  const std::complex<float>* samples,
                                  std::size_t num_samples)
{
    // TODO: Need a way to determine which band this is
}

} // namespace

reconstruct::sptr reconstruct::make(std::vector<band_spec> bands,
                                    const std::string& sample_format,
                                    bool zero_gaps,
                                    unsigned int compression_fft_size)
{
    return gnuradio::get_initial_sptr(
        new reconstruct_impl(bands, sample_format, zero_gaps, compression_fft_size));
}

/*
 * The private constructor
 */
reconstruct_impl::reconstruct_impl(const std::vector<band_spec>& bands,
                                   const std::string& sample_format,
                                   bool zero_gaps,
                                   unsigned int compression_fft_size)
    : gr::hier_block2(
          "reconstruct",
          // One input for compressed samples
          gr::io_signature::make(1, 1, sizeof(uint32_t)),
          // One output per band
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex)))
// Begin fields

{
    // TODO
    using namespace ::sparsdr;
    sparsdr_reconstruct_config* config =
        sparsdr_reconstruct_config_init(handle_reconstructed_samples, this);

    // Common config fields other than bands
    config->compression_fft_size = compression_fft_size;
    // TODO use zero_gaps
    

    if (sample_format == "N210 v1") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V1_N210;
        config->compressed_bandwidth = 100e6f;
    } else if (sample_format == "N210 v2") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V2;
        config->compressed_bandwidth = 100e6f;
    } else if (sample_format == "Pluto v1") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO;
        config->compressed_bandwidth = 61.44e6f;
    } else if (sample_format == "Pluto v2") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V2;
        config->compressed_bandwidth = 61.44e6f;
    } else {
        sparsdr_reconstruct_config_free(config);
        throw std::runtime_error("Unsupported sample format");
    }
    
    // Bands
    std::vector<sparsdr_reconstruct_band> c_bands;
    c_bands.reserve(bands.size());
    for (const band_spec& band : bands) {
        sparsdr_reconstruct_band c_band;
        c_band.frequency_offset = band.frequency();
        c_band.bins = band.bins();
        
        c_bands.push_back(c_band);
    }
    config->bands_length = c_bands.size();
    config->bands = c_bands.data();

    // Now that the context has been created, we can destroy the context
    sparsdr_reconstruct_config_free(config);
}

/*
 * Our virtual destructor.
 */
reconstruct_impl::~reconstruct_impl()
{
    // TODO
}

} /* namespace sparsdr */
} /* namespace gr */
