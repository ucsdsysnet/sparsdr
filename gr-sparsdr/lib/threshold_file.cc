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

#include "threshold_file.h"

#include <algorithm>
#include <fstream>
#include <sstream>

namespace gr {
namespace sparsdr {
threshold_file threshold_file::from_file(const std::string& path, std::size_t fft_size)
{
    std::ifstream file;
    // Throw exception if open fails
    file.exceptions(std::ios::badbit | std::ios::failbit);
    file.open(path);

    return threshold_file::from_stream(file, fft_size);
}

threshold_file threshold_file::from_stream(std::istream& stream, std::size_t fft_size)
{

    threshold_file result;
    result.thresholds.resize(fft_size, 0);
    bool have_gain = false;
    bool have_shift_amount = false;
    std::vector<bool> have_threshold(fft_size, false);

    // Throw exceptions when some things go wrong
    stream.exceptions(std::ios::badbit);

    std::string line;
    while (stream && !stream.eof()) {
        std::getline(stream, line);

        if (line.empty()) {
            break;
        }

        std::istringstream line_stream(line);
        line_stream.exceptions(std::ios::badbit | std::ios::failbit);

        std::string line_key;
        line_stream >> line_key;
        float line_value;
        line_stream >> line_value;

        // See if the first part of this line is a bin index
        std::istringstream line_key_stream(line_key);
        std::size_t bin_number;
        line_key_stream >> bin_number;
        if (line_key_stream) {
            // Bin index
            if (bin_number >= fft_size) {
                throw std::runtime_error("Bin number too large");
            }
            std::vector<bool>::reference have_this_threshold =
                have_threshold.at(bin_number);
            if (!have_this_threshold) {
                have_this_threshold = true;
                result.thresholds.at(bin_number) = static_cast<std::uint32_t>(line_value);
            } else {
                throw std::runtime_error("Duplicate bin number");
            }
        } else {
            // Other value
            if (line_key == "RxGaindB") {
                have_gain = true;
                result.gain = static_cast<std::int32_t>(line_value);
            } else if (line_key == "SuggestedShift") {
                have_shift_amount = true;
                result.shift_amount = static_cast<std::uint8_t>(line_value);
            } else {
                // ignore
            }
        }
    }
    // Reached end of file
    const bool complete = have_gain && have_shift_amount &&
                          std::all_of(have_threshold.cbegin(),
                                      have_threshold.cend(),
                                      [](bool have) { return have; });
    if (complete) {
        return result;
    } else {
        throw std::runtime_error("Incomplete file");
    }
}

} // namespace sparsdr
} // namespace gr
