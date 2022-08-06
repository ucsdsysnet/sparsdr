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

#include <memory>

#include "reconstruct_impl.h"
#include <gnuradio/io_signature.h>

#include "reconstruct_source_impl.h"
#include "reconstruct_sink.h"

namespace gr {
namespace sparsdr {


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
    : gr::hier_block2()
{
    using namespace ::sparsdr;
    sparsdr_reconstruct_config* config = sparsdr_reconstruct_config_init();

    // Common config fields other than bands
    config->compression_fft_size = compression_fft_size;
    config->zero_gaps = zero_gaps;


    int parser_sample_bytes;
    if (sample_format == "N210 v1") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V1_N210;
        config->compressed_bandwidth = 100e6f;
        parser_sample_bytes = 8;
    } else if (sample_format == "N210 v2") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V2;
        config->compressed_bandwidth = 100e6f;
        parser_sample_bytes = 4;
    } else if (sample_format == "Pluto v1") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO;
        config->compressed_bandwidth = 61.44e6f;
        parser_sample_bytes = 8;
    } else if (sample_format == "Pluto v2") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V2;
        config->compressed_bandwidth = 61.44e6f;
        parser_sample_bytes = 4;
    } else {
        sparsdr_reconstruct_config_free(config);
        throw std::runtime_error("Unsupported sample format");
    }

    // Bands (also create a source block for each one)
    std::vector<sparsdr_reconstruct_band> c_bands;
    c_bands.reserve(bands.size());
    for (std::size_t i = 0; i < bands.size(); i++) {
        const band_spec& band = bands.at(i);
        sparsdr_reconstruct_band c_band;
        c_band.frequency_offset = band.frequency();
        c_band.bins = band.bins();
        // For each band, call the single callback. Give it a context that the
        // source block also uses
        c_band.output_callback = reconstruct_source_impl::handle_reconstructed_samples;

        std::unique_ptr<output_context> context_ptr(new output_context);
        c_band.output_context = reinterpret_cast<void*>(context_ptr.get());

        c_bands.push_back(c_band);

        // Make source block and connect to output
        auto source_block = reconstruct_source::make(std::move(context_ptr));
        connect(source_block, 0, self(), i);
    }
    config->bands_length = c_bands.size();
    config->bands = c_bands.data();

    // Start reconstruction
    sparsdr_reconstruct_context* context;
    const int status = sparsdr_reconstruct_init(&context, config);
    // Now that the context has been created, we can destroy the config
    sparsdr_reconstruct_config_free(config);
    if (status != SPARSDR_RECONSTRUCT_OK) {
        std::stringstream stream;
        stream << "sparsdr_reconstruct_init returned " << status;
        throw std::runtime_error(stream.str());
    }

    // Make and connect sink block
    // Context ownership moves into the sink block
    auto sink_block = reconstruct_sink::make(context, parser_sample_bytes);
    connect(self(), 0, sink_block, 0);
}

/*
 * Our virtual destructor.
 */
reconstruct_impl::~reconstruct_impl() {}

} /* namespace sparsdr */
} /* namespace gr */
