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

#include "stream_average_model.h"

namespace gr {
namespace sparsdr {

stream_average_model::stream_average_model(std::size_t capacity)
    : _rows(), _capacity(capacity), _last_index(0)
{
}

void stream_average_model::store_sample(std::uint16_t index, std::uint32_t average)
{
    if (_rows.empty()) {
        // Special case - first sample
        _rows.emplace_front();
        std::fill(_rows.front().begin(), _rows.front().end(), 0);
        _rows.front().at(index) = average;
    } else if (index < _last_index) {
        // Next row
        if (_rows.size() == _capacity) {
            _rows.pop_back();
        }
        _rows.emplace_front();
        std::fill(_rows.front().begin(), _rows.front().end(), 0);
        _rows.front().at(index) = average;
    }
    // Set the value
    _rows.front().at(index) = average;
    _last_index = index;
}

std::size_t stream_average_model::size() const { return _rows.size(); }

const std::uint32_t* stream_average_model::averages(std::size_t index) const
{
    return _rows.at(index).data();
}

} // namespace sparsdr
} // namespace gr
