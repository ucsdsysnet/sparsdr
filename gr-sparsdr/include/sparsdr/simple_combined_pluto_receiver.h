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


#ifndef INCLUDED_SPARSDR_SIMPLE_COMBINED_PLUTO_RECEIVER_H
#define INCLUDED_SPARSDR_SIMPLE_COMBINED_PLUTO_RECEIVER_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>
#include <sparsdr/simple_band_spec.h>
#include <cstdint>
#include <string>
#include <vector>

namespace gr {
namespace sparsdr {

/*!
 * \brief A wrapper of a combined_pluto_receive that can be configured using
 * frequency ranges, without manually calculating bins
 * \ingroup sparsdr
 *
 */
class SPARSDR_API simple_combined_pluto_receiver : virtual public gr::hier_block2
{
public:
    typedef boost::shared_ptr<simple_combined_pluto_receiver> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of
     * sparsdr::simple_combined_pluto_receiver.
     *
     * To avoid accidental use of raw pointers, sparsdr::simple_combined_pluto_receiver's
     * constructor is in a private implementation
     * class. sparsdr::simple_combined_pluto_receiver::make is the public interface for
     * creating new instances.
     *
     * \param uri The IIO context URI to use when connecting to the Pluto
     * \param buffer_size The size of the IIO buffers, in 16-bit samples
     * \param center_frequency The center frequency to tune to, in hertz
     * \param bands The bands to receive and reconstruct (all these frequencies
     *   are absolute)
     * \param threshold The threshold to apply to all unmasked bins
     * \param zero_gaps True if zero samples should be included in time gaps
     *   in the outputs
     */
    static sptr make(const std::string& uri,
                     std::size_t buffer_size,
                     float center_frequency,
                     const std::vector<simple_band_spec>& bands,
                     std::uint32_t threshold,
                     bool zero_gaps = false,
                     bool skip_bin_config = false);

    // Functions that delegate to combined_pluto_receiver functions
    virtual void set_gain(double gain) = 0;
    /**
     * Sets the gain control mode, which can be "manual" or
     * an automatic gain control mode
     */
    virtual void set_gain_control_mode(const std::string& mode) = 0;
    virtual void set_shift_amount(std::uint8_t scaling) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_SIMPLE_COMBINED_PLUTO_RECEIVER_H */
