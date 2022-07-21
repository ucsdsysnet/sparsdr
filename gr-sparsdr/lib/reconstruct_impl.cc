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


    if (sample_format == "N210 v1") {
        // arguments.push_back("--compressed-bandwidth");
        // arguments.push_back("100e6");
        // arguments.push_back("--sample-format");
        // arguments.push_back("v1-n210");
        // arguments.push_back("--timestamp-bits");
        // arguments.push_back("20");
    } else if (sample_format == "N210 v2") {
        // arguments.push_back("--compressed-bandwidth");
        // arguments.push_back("100e6");
        // arguments.push_back("--sample-format");
        // arguments.push_back("v2");
        // arguments.push_back("--timestamp-bits");
        // arguments.push_back("30");
    } else if (sample_format == "Pluto v1") {
        // arguments.push_back("--compressed-bandwidth");
        // arguments.push_back("61.44e6");
        // arguments.push_back("--sample-format");
        // arguments.push_back("v1-pluto");
        // arguments.push_back("--timestamp-bits");
        // arguments.push_back("21");
    } else if (sample_format == "Pluto v2") {
        // arguments.push_back("--compressed-bandwidth");
        // arguments.push_back("61.44e6");
        // arguments.push_back("--sample-format");
        // arguments.push_back("v2");
        // arguments.push_back("--timestamp-bits");
        // arguments.push_back("30");
    } else {
        sparsdr_reconstruct_config_free(config);
        throw std::runtime_error("Unsupported sample format");
    }

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
