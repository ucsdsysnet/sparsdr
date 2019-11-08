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

#include "average_model.h"

namespace gr {
namespace sparsdr {

std::uint32_t AverageModel::max() const {
    std::uint32_t max = 0;
    for (std::size_t i = 0; i < size(); i++) {
        const auto row = averages(i);
        for (int j = 0; j < 2048; j++) {
            if (row[j] > max) {
                max = row[j];
            }
        }
    }
    return max;
}

}
}
