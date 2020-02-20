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

#ifndef INCLUDED_SPARSDR_TAGGED_WAVFILE_SINK_IMPL_H
#define INCLUDED_SPARSDR_TAGGED_WAVFILE_SINK_IMPL_H

#include <sparsdr/tagged_wavfile_sink.h>
#include <cstdio>

namespace gr {
  namespace sparsdr {

    class tagged_wavfile_sink_impl : public tagged_wavfile_sink
    {
     private:
      /** The path to the directory where files should be written */
      std::string d_directory;
      /** Sample rate, samples/second */
      unsigned int d_sample_rate;
      /** Bits used for each sample */
      unsigned int d_bits_per_sample;
      /** The WAV file currently open and being written */
      FILE* d_current_file;
      /** Number of bytes of samples written to d_current_file */
      unsigned int d_bytes_written;

     public:
      tagged_wavfile_sink_impl(const std::string& directory, unsigned int sample_rate, int bits_per_sample);
      ~tagged_wavfile_sink_impl();

      // Where all the action really happens
      int work(int noutput_items,
         gr_vector_const_void_star &input_items,
         gr_vector_void_star &output_items);
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_TAGGED_WAVFILE_SINK_IMPL_H */
