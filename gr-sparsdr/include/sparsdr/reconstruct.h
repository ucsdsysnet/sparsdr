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


#ifndef INCLUDED_SPARSDR_RECONSTRUCT_H
#define INCLUDED_SPARSDR_RECONSTRUCT_H

#include <gnuradio/block.h>
#include <sparsdr/api.h>
#include <sparsdr/band_spec.h>

namespace gr {
namespace sparsdr {

/*!
 * \brief The SparSDR reconstruct block receives compressed samples
 * and reconstructs signals from one or more bands
 * \ingroup sparsdr
 *
 */
class SPARSDR_API reconstruct : virtual public gr::block
{
public:
    typedef boost::shared_ptr<reconstruct> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::reconstruct.
     *
     * To avoid accidental use of raw pointers, sparsdr::reconstruct's
     * constructor is in a private implementation
     * class. sparsdr::reconstruct::make is the public interface for
     * creating new instances.
     *
     * \param bands the bands to decompress
     * \param sample_format The compressed sample format and source device
     * (this should be "N210 v1", "N210 v2", "Pluto v1", or "Pluto v2")
     * \param zero_gaps true to insert zero samples in the output(s) for periods
     * when there were no active signals
     * \param compression_fft_size the number of bins in the FFT used to
     * compress the received signals
     */
    static sptr make(std::vector<::gr::sparsdr::band_spec> bands,
                     const std::string& sample_format = "N210 v1",
                     bool zero_gaps = false,
                     unsigned int compression_fft_size = 1024);
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_H */
