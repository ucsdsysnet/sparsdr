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

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <gnuradio/io_signature.h>
#include "reconstruct_source_impl.h"

namespace gr {
  namespace sparsdr {

    reconstruct_source::sptr
    reconstruct_source::make()
    {
      return gnuradio::get_initial_sptr
        (new reconstruct_source_impl());
    }


    /*
     * The private constructor
     */
    reconstruct_source_impl::reconstruct_source_impl()
      : gr::sync_block("reconstruct_source",
              gr::io_signature::make(0, 0, 0),
              gr::io_signature::make(1, 1, sizeof(gr_complex)))
    {}

    /*
     * Our virtual destructor.
     */
    reconstruct_source_impl::~reconstruct_source_impl()
    {
    }

    int
    reconstruct_source_impl::work(int noutput_items,
        gr_vector_const_void_star &input_items,
        gr_vector_void_star &output_items)
    {
      // TODO
      // Tell runtime system how many output items we produced.
      return noutput_items;
    }

  } /* namespace sparsdr */
} /* namespace gr */

