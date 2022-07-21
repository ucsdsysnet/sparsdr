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
    : gr::block("reconstruct",
                // One input for compressed samples
                gr::io_signature::make(1, 1, sizeof(uint32_t)),
                // One output per band
                gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex))),
      // Begin fields
      d_output_contexts(make_output_contexts(this, bands.size()))
{
    using namespace ::sparsdr;
    sparsdr_reconstruct_config* config = sparsdr_reconstruct_config_init();

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
    for (std::size_t i = 0; i < bands.size(); i++) {
        const band_spec& band = bands.at(i);
        sparsdr_reconstruct_band c_band;
        c_band.frequency_offset = band.frequency();
        c_band.bins = band.bins();
        // For each band, call the single callback. Give it a context that points
        // to this block and gives the band number.
        c_band.output_callback = reconstruct_impl::handle_reconstructed_samples;
        c_band.output_context =
            const_cast<void*>(reinterpret_cast<const void*>(&d_output_contexts.at(i)));

        c_bands.push_back(c_band);
    }
    config->bands_length = c_bands.size();
    config->bands = c_bands.data();

    // Now that the context has been created, we can destroy the context
    sparsdr_reconstruct_config_free(config);
}

/**
 * Generates a vector of output_context objects with successive band index values starting
 * at 0
 */
std::vector<reconstruct_impl::output_context>
reconstruct_impl::make_output_contexts(reconstruct_impl* reconstruct, std::size_t count)
{
    std::vector<output_context> contexts;
    for (std::size_t i = 0; i < count; i++) {
        output_context context;
        context.reconstruct = reconstruct;
        context.band_index = i;
        contexts.push_back(context);
    }
    return contexts;
}

int reconstruct_impl::general_work(int noutput_items,
                                   gr_vector_int& ninput_items,
                                   gr_vector_const_void_star& input_items,
                                   gr_vector_void_star& output_items)
{
    // TODO
    return 0;
}


/**
 * The reconstruction library calls this function (potentially from many
 * different threads) when it has produced samples
 */
void reconstruct_impl::handle_reconstructed_samples(void* context,
                                                    const std::complex<float>* samples,
                                                    std::size_t num_samples)
{
    const output_context* output_context =
        reinterpret_cast<reconstruct_impl::output_context*>(context);
    // TODO
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
