/* -*- c++ -*- */
/*
 * Copyright 2019 The Regents of the University of California.
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

#ifndef INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_IMPL_H
#define INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_IMPL_H

#include <sparsdr/sample_distributor.h>
#include <vector>
#include <atomic>

namespace gr {
  namespace sparsdr {

    class sample_distributor_impl : public sample_distributor
    {
     private:

      /**
       * Information about a decoder that this sample distributor can supply
       * with samples
       */
      class decoder_info
      {
      public:
        /** Special value that indicates that this decoder is not in use */
        static const int NO_INPUT = -1;
        /** The index of the input that is using this decoder */
        int d_input;
      };

      /** The size of stream items this block processes */
      int d_item_size;

      /**
       * The decoders available for this block to use
       *
       * Each index in this vector is also an output index for this block.
       *
       * Thread safety: Access only when holding a lock on d_setlock
       */
      std::vector<decoder_info> d_decoders;

      /**
       * The number of decoders this block has available but did not use
       * in the last call to general_work()
       *
       * A negative value means that not enough decoders are available for the
       * number of active inputs.
       */
      std::atomic_int d_decoder_surplus;

     public:
      sample_distributor_impl(int item_size);
      ~sample_distributor_impl();

      // Where all the action really happens
      void forecast (int noutput_items, gr_vector_int &ninput_items_required);

      int general_work(int noutput_items,
           gr_vector_int &ninput_items,
           gr_vector_const_void_star &input_items,
           gr_vector_void_star &output_items);

      virtual int decoder_surplus() const override;
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_IMPL_H */
