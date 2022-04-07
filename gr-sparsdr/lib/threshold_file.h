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
#ifndef SPARSDR_THRESHOLD_FILE_H
#define SPARSDR_THRESHOLD_FILE_H

#include <sparsdr/api.h>
#include <cstdint>
#include <istream>
#include <string>
#include <vector>

namespace gr {
namespace sparsdr {

/**
 * Thresholds and other information read from a file
 */
// Although this is not part of the public API,
// it is annoted with SPARSDR_API so the testing code can link to it.
struct SPARSDR_API threshold_file {
public:
    /** Gain, from the RxGaindB line */
    std::int32_t gain;
    /** Shift amount, from the SuggestedShift line */
    std::uint8_t shift_amount;
    /** A threshold for each bin */
    std::vector<std::uint32_t> thresholds;

    /**
     * Reads information from a file at the provided path
     *
     * This function throws an unspecified exception if an error occurs.
     *
     * @param path the path to the file
     * @param fft_size the number of bins in the compression FFT
     *        (this determines the number of threshold values returned)
     */
    static threshold_file from_file(const std::string& path, std::size_t fft_size);
    /**
     * Reads information from the provided stream
     *
     * This function throws an unspecified exception if an error occurs.
     *
     * @param path the path to the file
     * @param fft_size the number of bins in the compression FFT
     *        (this determines the number of threshold values returned)
     */
    static threshold_file from_stream(std::istream& stream, std::size_t fft_size);
};

} // namespace sparsdr
} // namespace gr

#endif
