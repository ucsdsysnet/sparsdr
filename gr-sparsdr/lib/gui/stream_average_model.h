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

#ifndef INCLUDED_SPARSDR_STREAM_AVERAGE_MODEL_H
#define INCLUDED_SPARSDR_STREAM_AVERAGE_MODEL_H

#include <array>
#include <cstdint>
#include <deque>

#include "average_model.h"

namespace gr {
namespace sparsdr {

/**
 * An AverageModel that collects average values from a stream of samples
 */
class stream_average_model : public AverageModel
{
private:
    /**
     * Queue of rows
     *
     * Each row represents one set of 2048 average values, sent by the USRP
     * at about the same time. The front of the queue contains the newest values.
     */
    std::deque<std::array<std::uint32_t, 2048>> _rows;

    /**
     * Maximum number of rows to store
     */
    std::size_t _capacity;

    /**
     * The FFT index (0..2048) of the last sample received. This is used to
     * detect when a new row is beginning.
     */
    std::uint16_t _last_index;

public:
    stream_average_model(std::size_t capacity);

    /**
     * Stores a sample in this model, shifting rows as necessary
     */
    void store_sample(std::uint16_t index, std::uint32_t average);

    virtual std::size_t size() const override;

    virtual const std::uint32_t* averages(std::size_t index) const override;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_STREAM_AVERAGE_MODEL_H */
