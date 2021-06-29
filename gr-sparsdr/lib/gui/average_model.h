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

#ifndef AVERAGE_MODEL_H
#define AVERAGE_MODEL_H
#include <cstdint>

namespace gr {
namespace sparsdr {

/**
 * @brief The AverageModel class defines an interface that an
 * AverageWaterfallView uses to get averages to display.
 */
class AverageModel
{
public:
    /**
     * @return the number of sets of averages in this model
     */
    virtual std::size_t size() const = 0;

    /**
     * @brief averages gets a set of average values.
     * @param index The index to get averages for. This must be less than
     * this->size().
     * @return A pointer to 2048 uint32_t values representing the averages at
     * the requested index
     */
    virtual const std::uint32_t* averages(std::size_t index) const = 0;

    /**
     * @brief max returns the maximum average value in this model
     * @return the maximum value
     */
    virtual std::uint32_t max() const;

    virtual ~AverageModel() = default;
};

} // namespace sparsdr
} // namespace gr

#endif // AVERAGE_MODEL_H
