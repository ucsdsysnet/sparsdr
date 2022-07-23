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
#include <exception>
#include <iostream>
#include <sstream>
#include <stdexcept>

namespace gr {
namespace sparsdr {

namespace {

/**
 * Size of a GNU Radio input sample in bytes
 * (the size that the compressed sample parser uses may be different)
 */
constexpr std::size_t GR_IN_SAMPLE_BYTES = sizeof(std::uint32_t);

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
                gr::io_signature::make(1, 1, GR_IN_SAMPLE_BYTES),
                // One output per band
                gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex))),
      // Begin fields
      d_output_contexts(make_output_contexts(bands.size())),
      d_parser_sample_bytes(0), // Will be corrected below
      d_context(nullptr)        // Will be corrected below
{
    using namespace ::sparsdr;
    sparsdr_reconstruct_config* config = sparsdr_reconstruct_config_init();

    // Common config fields other than bands
    config->compression_fft_size = compression_fft_size;
    config->zero_gaps = zero_gaps;


    if (sample_format == "N210 v1") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V1_N210;
        config->compressed_bandwidth = 100e6f;
        d_parser_sample_bytes = 8;
    } else if (sample_format == "N210 v2") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V2;
        config->compressed_bandwidth = 100e6f;
        d_parser_sample_bytes = 4;
    } else if (sample_format == "Pluto v1") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V1_PLUTO;
        config->compressed_bandwidth = 61.44e6f;
        d_parser_sample_bytes = 8;
    } else if (sample_format == "Pluto v2") {
        config->format = SPARSDR_RECONSTRUCT_FORMAT_V2;
        config->compressed_bandwidth = 61.44e6f;
        d_parser_sample_bytes = 4;
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

        std::unique_ptr<output_context>& context_ptr = d_output_contexts.at(i);
        c_band.output_context = reinterpret_cast<void*>(context_ptr.get());

        c_bands.push_back(c_band);
    }
    config->bands_length = c_bands.size();
    config->bands = c_bands.data();

    // Start reconstruction
    const int status = sparsdr_reconstruct_init(&d_context, config);
    // Now that the context has been created, we can destroy the config
    sparsdr_reconstruct_config_free(config);
    if (status != SPARSDR_RECONSTRUCT_OK) {
        std::stringstream stream;
        stream << "sparsdr_reconstruct_init returned " << status;
        throw std::runtime_error(stream.str());
    }
}

/**
 * Generates a vector of output_context objects
 */
std::vector<std::unique_ptr<reconstruct_impl::output_context>>
reconstruct_impl::make_output_contexts(std::size_t count)
{
    std::vector<std::unique_ptr<output_context>> contexts;
    contexts.reserve(count);
    for (std::size_t i = 0; i < count; i++) {
        std::unique_ptr<output_context> context(new output_context);
        contexts.push_back(std::move(context));
    }
    return contexts;
}

int reconstruct_impl::general_work(int noutput_items,
                                   gr_vector_int& ninput_items,
                                   gr_vector_const_void_star& input_items,
                                   gr_vector_void_star& output_items)
{
    using namespace ::sparsdr;
    // Part 1: Input
    const std::size_t num_input_bytes = ninput_items[0] * GR_IN_SAMPLE_BYTES;
    const std::size_t input_compressed_samples = num_input_bytes / d_parser_sample_bytes;
    const std::uint8_t* input_bytes =
        reinterpret_cast<const std::uint8_t*>(input_items[0]);
    for (std::size_t i = 0; i < input_compressed_samples; i++) {
        const std::uint8_t* sample = input_bytes + (i * d_parser_sample_bytes);
        const int status = sparsdr_reconstruct_handle_samples(
            d_context, reinterpret_cast<const void*>(sample), d_parser_sample_bytes);
        if (status != 0) {
            std::cerr << "sparsdr_reconstruct_handle_samples returned " << status << '\n';
            return WORK_DONE;
        }
    }

    // Part 2: Outputs
    for (std::size_t i = 0; i < d_output_contexts.size(); i++) {
        std::unique_ptr<output_context>& output = d_output_contexts.at(i);
        void* raw_out_buffer = output_items.at(i);
        gr_complex* out_buffer = reinterpret_cast<gr_complex*>(raw_out_buffer);
        // Lock the mutex and copy some outputs
        std::unique_lock<std::mutex> lock(output->mutex);
        // Copy one complex value at a time until the queue is empty or noutput_items
        // values have been copied
        int items_copied = 0;
        auto done = [&] {
            return items_copied == noutput_items || output->queue.empty();
        };
        for (items_copied = 0; !done(); items_copied++) {
            *out_buffer = output->queue.front();
            out_buffer += 1;
            output->queue.pop();
        }
        // Tell the scheduler the number of items copied for this output
        produce(i, items_copied);
    }

    return WORK_CALLED_PRODUCE;
}


/**
 * The reconstruction library calls this function (potentially from many
 * different threads) when it has produced samples
 */
void reconstruct_impl::handle_reconstructed_samples(void* context,
                                                    const std::complex<float>* samples,
                                                    std::size_t num_samples)
{
    try {
        output_context* output_context =
            reinterpret_cast<reconstruct_impl::output_context*>(context);
        // Lock the mutex and copy the samples to the queue
        std::unique_lock<std::mutex> lock(output_context->mutex);
        for (std::size_t i = 0; i < num_samples; i++) {
            output_context->queue.push(samples[i]);
        }
    } catch (std::exception& e) {
        // Don't let C++ exceptions propagate into Rust
        std::cerr << "Unexpected exception in reconstructed sample callback: " << e.what()
                  << '\n';
        std::terminate();
    } catch (...) {
        std::cerr << "Unexpected unknown exception in reconstructed sample callback\n";
        std::terminate();
    }
}

/*
 * Our virtual destructor.
 */
reconstruct_impl::~reconstruct_impl()
{
    ::sparsdr::sparsdr_reconstruct_free(d_context);
    d_context = nullptr;
}

} /* namespace sparsdr */
} /* namespace gr */
