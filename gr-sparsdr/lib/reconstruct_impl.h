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

#ifndef INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H
#define INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H

#include <sparsdr/reconstruct.h>
#include <unistd.h>
#include <boost/noncopyable.hpp>
#include <complex>
#include <memory>
#include <mutex>
#include <queue>
#include <vector>

#include <sparsdr_reconstruct.hpp>

namespace gr {
namespace sparsdr {

// Inherit from noncopyable to prevent copying d_child
class reconstruct_impl : public reconstruct, public boost::noncopyable
{
private:
    /** A value used with the output callback as a context */
    struct output_context {
        /** Mutex used to protect queue */
        std::mutex mutex;
        /** Queue of samples produced by the reconstruction library */
        std::queue<std::complex<float>> queue;
    };

    /**
     * Allocated output contexts
     * (this must not be changed while the reconstruction context exists)
     */
    std::vector<std::unique_ptr<output_context>> d_output_contexts;
    /** Number of bytes the parser expects in every compressed sample */
    std::size_t d_parser_sample_bytes;
    /** Reconstruction context */
    ::sparsdr::sparsdr_reconstruct_context* d_context;

    /** Callback that handles reconstructed samples */
    static void handle_reconstructed_samples(void* context,
                                             const std::complex<float>* samples,
                                             std::size_t num_samples);

    /**
     * Generates a vector of output_context objects
     */
    static std::vector<std::unique_ptr<output_context>>
    make_output_contexts(std::size_t count);

public:
    reconstruct_impl(const std::vector<band_spec>& bands,
                     const std::string& sample_format,
                     bool zero_gaps,
                     unsigned int compression_fft_size);

    int general_work(int noutput_items,
                     gr_vector_int& ninput_items,
                     gr_vector_const_void_star& input_items,
                     gr_vector_void_star& output_items);

    ~reconstruct_impl();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H */
