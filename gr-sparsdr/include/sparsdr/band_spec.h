/* -*- c++ -*- */
/*
 * Copyright 2019 The Regents of the University of California
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


#ifndef INCLUDED_SPARSDR_BAND_SPEC_H
#define INCLUDED_SPARSDR_BAND_SPEC_H
#include <cstdint>

namespace gr {
namespace sparsdr {

/*!
 * \brief A specification of a band to reconstruct
 */
class band_spec
{
private:
    /*!
     * \brief The frequency to decompress,
     *
     * When used with a reconstruct block, this frequency is relative to the
     * center frequency of the original capture.
     *
     * When used with a combined receiver block, this frequency is absolute.
     */
    float d_frequency;
    /*! \brief the number of bins to decompress */
    uint16_t d_bins;

public:
    /*!
     * \brief creates a band specification
     *
     * \param frequency The frequency to decompress, in hertz relative to the
     * center frequency of the compressed capture
     *
     * When used with a reconstruct block, this frequency is relative to the
     * center frequency of the original capture.
     *
     * When used with a combined receiver block, this frequency is absolute.
     *
     * \param bins the number of bins to decompress
     */
    inline band_spec(float frequency, uint16_t bins)
        : d_frequency(frequency), d_bins(bins)
    {
    }

    /*!
     * \brief Creates a band specification with frequency and bins both set
     * to zero
     */
    inline band_spec() : d_frequency(0.0), d_bins(0) {}


    inline float frequency() const { return d_frequency; }
    inline uint16_t bins() const { return d_bins; }
};

} // namespace sparsdr
} // namespace gr

#endif
