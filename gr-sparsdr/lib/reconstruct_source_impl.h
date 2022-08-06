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

#ifndef INCLUDED_SPARSDR_RECONSTRUCT_SOURCE_IMPL_H
#define INCLUDED_SPARSDR_RECONSTRUCT_SOURCE_IMPL_H

#include <memory>

#include "reconstruct_source.h"

namespace gr {
namespace sparsdr {

class reconstruct_source_impl : public reconstruct_source
{
private:
    /** Context used by the reconstructed sample callback */
    std::unique_ptr<output_context> d_context;

public:
    reconstruct_source_impl(std::unique_ptr<output_context>&& context);
    ~reconstruct_source_impl();

    // Where all the action really happens
    int work(int noutput_items,
             gr_vector_const_void_star& input_items,
             gr_vector_void_star& output_items);


    /** Callback that handles reconstructed samples */
    static void handle_reconstructed_samples(void* context,
                                             const std::complex<float>* samples,
                                             std::size_t num_samples);
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_SOURCE_IMPL_H */
