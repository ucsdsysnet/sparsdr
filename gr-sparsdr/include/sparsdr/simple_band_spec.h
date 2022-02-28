/* -*- c++ -*- */
/*
 * Copyright 2022 The Regents of the University of California
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


#ifndef INCLUDED_SPARSDR_SIMPLE_BAND_SPEC_H
#define INCLUDED_SPARSDR_SIMPLE_BAND_SPEC_H

namespace gr {
namespace sparsdr {

/*!
 * \brief A specification of a band to receive and reconstruct
 */
class simple_band_spec
{
private:
    /*!
     * \brief The center frequency to receive
     *
     * This frequency is absolute and in hertz.
     */
    float d_frequency;
    /*! \brief the width of this band in hertz */
    float d_bandwidth;

public:
    /*!
     * \brief creates a band specification
     *
     * \param frequency The center frequency to receive
     * \param bandwidth the width of the band to receive
     */
    inline simple_band_spec(float frequency, float bandwidth)
        : d_frequency(frequency), d_bandwidth(bandwidth)
    {
    }

    /*!
     * \brief Creates a band specification with frequency and bandwidth both set
     * to zero
     */
    inline simple_band_spec() : d_frequency(0.0), d_bandwidth(0.0) {}


    inline float frequency() const { return d_frequency; }
    inline float bandwidth() const { return d_bandwidth; }
};

} // namespace sparsdr
} // namespace gr

#endif
