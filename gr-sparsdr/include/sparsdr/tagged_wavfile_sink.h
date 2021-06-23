/* -*- c++ -*- */
/*
 * Copyright 2020 The Regents of the University of California.
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


#ifndef INCLUDED_SPARSDR_TAGGED_WAVFILE_SINK_H
#define INCLUDED_SPARSDR_TAGGED_WAVFILE_SINK_H

#include <gnuradio/sync_block.h>
#include <sparsdr/api.h>

namespace gr {
namespace sparsdr {

/*!
 * \brief Writes audio from a stream to multiple WAV files. A stream tag
 * triggers a new file.
 * \ingroup sparsdr
 *
 */
class SPARSDR_API tagged_wavfile_sink : virtual public gr::sync_block
{
public:
    typedef boost::shared_ptr<tagged_wavfile_sink> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::tagged_wavfile_sink.
     *
     * To avoid accidental use of raw pointers, sparsdr::tagged_wavfile_sink's
     * constructor is in a private implementation
     * class. sparsdr::tagged_wavfile_sink::make is the public interface for
     * creating new instances.
     *
     * @param directory the path to the directory to put the files
     * @param sample_rate the sample rate to write
     * @param bits_per_sample the number of bits to use for each sample
     */
    static sptr make(const std::string& directory,
                     unsigned int sample_rate,
                     int bits_per_sample = 16);
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_TAGGED_WAVFILE_SINK_H */
