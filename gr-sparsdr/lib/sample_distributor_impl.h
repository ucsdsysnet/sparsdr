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

#ifndef INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_IMPL_H
#define INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_IMPL_H

#include <sparsdr/sample_distributor.h>
#include <atomic>
#include <vector>

#include "named_pipe_reader.h"

namespace gr {
namespace sparsdr {

class sample_distributor_impl : public sample_distributor
{
private:
    struct input_info {
        static const int NO_OUTPUT = -1;
        /** The index of the output (decoder) that is connected to ths input */
        int d_output;

        inline input_info() : d_output(NO_OUTPUT) {}
    };

    named_pipe_reader d_pipe_reader;

    /** The size of stream items this block processes */
    int d_item_size;

    /**
     * Inputs (named pipes) that this block reads from
     *
     * Thread safety: Access only from the general_work function in the
     * block thread
     */
    std::vector<input_info> d_inputs;

    /**
     * A value for each output. True indicates that the output is claimed
     * by an input
     */
    std::vector<bool> d_outputs_used;

    /**
     * The number of decoders this block has available but did not use
     * in the last call to general_work()
     *
     * A negative value means that not enough decoders are available for the
     * number of active inputs.
     */
    std::atomic_int d_decoder_surplus;

    /**
     * Adds a stream tag to the next output sample, specifying that the sample
     * came from a particular source
     *
     * @param in_index The index of the input where the sample came in
     * @param out_index The index of the output where the sample and the
     * associated tag should go out
     */
    void add_source_tag(int in_index, int out_index);

public:
    sample_distributor_impl(int item_size, const std::vector<std::string>& pipe_paths);
    ~sample_distributor_impl();

    void forecast(int noutput_items, gr_vector_int& ninput_items_required);

    int general_work(int noutput_items,
                     gr_vector_int& ninput_items,
                     gr_vector_const_void_star& input_items,
                     gr_vector_void_star& output_items);

    virtual int decoder_surplus() const override;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_IMPL_H */
