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

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <gnuradio/io_signature.h>
#include <gnuradio/blocks/wavfile.h>
#include "tagged_wavfile_sink_impl.h"

namespace gr {
  namespace sparsdr {

    tagged_wavfile_sink::sptr
    tagged_wavfile_sink::make(const std::string& directory, unsigned int sample_rate, int bits_per_sample)
    {
      return gnuradio::get_initial_sptr
        (new tagged_wavfile_sink_impl(directory, sample_rate, bits_per_sample));
    }

    /*
     * The private constructor
     */
    tagged_wavfile_sink_impl::tagged_wavfile_sink_impl(const std::string& directory, unsigned int sample_rate, int bits_per_sample)
      : gr::sync_block("tagged_wavfile_sink",
              gr::io_signature::make(1, 1, sizeof(float)),
              gr::io_signature::make(0, 0, 0)),
        d_directory(directory),
        d_sample_rate(sample_rate),
        d_bits_per_sample(bits_per_sample),
        d_current_file(nullptr),
        d_bytes_written(0)
    {}

    /*
     * Our virtual destructor.
     */
    tagged_wavfile_sink_impl::~tagged_wavfile_sink_impl()
    {
        if (d_current_file != nullptr) {
            // Finish writing and close the file
            gr::blocks::wavheader_complete(d_current_file, d_bytes_written);
            // TODO: Error handling (above and below)
            std::fclose(d_current_file);
        }
    }

    int
    tagged_wavfile_sink_impl::work(int noutput_items,
        gr_vector_const_void_star &input_items,
        gr_vector_void_star &output_items)
    {
      const float* in = static_cast<const float*>(input_items[0]);

      // Do <+signal processing+>

      // Tell runtime system how many output items we produced.
      return noutput_items;
    }

  } /* namespace sparsdr */
} /* namespace gr */
