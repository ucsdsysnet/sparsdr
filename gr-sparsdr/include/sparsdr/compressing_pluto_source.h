/* -*- c++ -*- */
/*
 * Copyright 2021-2022 The Regents of the University of California.
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

#ifndef INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_H
#define INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>
#include <sparsdr/compressing_source.h>
#include <cstdint>

namespace gr {
namespace sparsdr {

/*!
 * \brief Connects to a suitably configured ADALM-Pluto radio and reads
 * compressed samples
 *
 * \ingroup sparsdr
 *
 */
class SPARSDR_API compressing_pluto_source : virtual public gr::hier_block2,
                                             public compressing_source
{
public:
    typedef boost::shared_ptr<compressing_pluto_source> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of
     * sparsdr::compressing_pluto_source.
     *
     * To avoid accidental use of raw pointers,
     * sparsdr::compressing_pluto_source's constructor is in a private
     * implementation class. sparsdr::compressing_pluto_source::make is the public
     * interface for creating new instances.
     *
     * \param uri The URI to use to create the IIO context. Example value:
     * `ip:192.168.2.1`
     * \param buffer_size The size of compressed sample buffers, in 32-bit
     * samples. Values that are too small may cause silent overflow and data loss.
     *
     */
    static sptr make(const std::string& uri, std::size_t buffer_size = 1024 * 1024);

    /**
     * Returns the SparSDR compressed sample format version that this device
     * sends
     */
    virtual unsigned int format_version() const = 0;

    // Front-end settings

    /**
     * Sets the center frequency to receive
     *
     * \param frequency the center frequency in hertz
     */
    virtual void set_frequency(unsigned long long frequency) = 0;

    /**
     * Sets the receive gain (for manual gain control mode)
     *
     * \param gain the gain in decibels
     */
    virtual void set_gain(double gain) = 0;

    /**
     * Sets the gain control mode, which can be "manual" or
     * an automatic gain control mode
     */
    virtual void set_gain_control_mode(const std::string& mode) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_H */
