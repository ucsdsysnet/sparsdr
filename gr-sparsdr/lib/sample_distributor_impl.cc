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

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <gnuradio/io_signature.h>
#include "sample_distributor_impl.h"

namespace gr {
  namespace sparsdr {

    sample_distributor::sptr
    sample_distributor::make(int inputs)
    {
      return gnuradio::get_initial_sptr
        (new sample_distributor_impl(inputs));
    }

    /*
     * The private constructor
     */
    sample_distributor_impl::sample_distributor_impl(int item_size)
      : gr::block("sample_distributor",
              // Any number of inputs
              gr::io_signature::make(0, gr::io_signature::IO_INFINITE, item_size),
              // Any number of outputs
              gr::io_signature::make(0, gr::io_signature::IO_INFINITE, item_size)),
        d_item_size(item_size),
        d_decoders(),
        d_decoder_surplus(0)
    {}

    /*
     * Our virtual destructor.
     */
    sample_distributor_impl::~sample_distributor_impl()
    {
    }

    void
    sample_distributor_impl::forecast (int noutput_items, gr_vector_int &ninput_items_required)
    {
      /* <+forecast+> e.g. ninput_items_required[0] = noutput_items */

      // Initialize: No input items required. We'll take whatever items we
      // can get, on any input channel.
      std::fill(ninput_items_required.begin(), ninput_items_required.end(), 0);
    }

    int
    sample_distributor_impl::general_work (int noutput_items,
                       gr_vector_int &ninput_items,
                       gr_vector_const_void_star &input_items,
                       gr_vector_void_star &output_items)
    {
      // noutput_items: Maximum number of items to write to each output
      // ninput_items: Number of items available to read from the various
      //     inputs

      // const <+ITYPE+> *in = (const <+ITYPE+> *) input_items[0];
      // <+OTYPE+> *out = (<+OTYPE+> *) output_items[0];

      // Keep track of the decoder surplus in this call to general_work()
      int local_decoder_surplus = 0;

      // Lock mutex to access d_decoders
      gr::thread::scoped_lock encoders_lock(d_setlock);

      // If any decoder is being used for an input that has no samples,
      // disassociate the input from the decoder and make it available again
      for (decoder_info& decoder : d_decoders) {
          if (decoder.d_input != decoder_info::NO_INPUT) {
              if (ninput_items.at(decoder.d_input) == 0) {
                  decoder.d_input = decoder_info::NO_INPUT;
                  local_decoder_surplus += 1;
              }
          }
      }

      // Copy items across each connection
      for (int out_index = 0; out_index < d_decoders.size(); out_index++) {
          const decoder_info& decoder = d_decoders[out_index];
          if (decoder.d_input != decoder_info::NO_INPUT) {
              const int in_index = decoder.d_input;

              // Calculate the number of items to process
              const int item_count = std::min(ninput_items.at(in_index), noutput_items);

              const void* input = input_items.at(in_index);
              void* output = output_items.at(out_index);

              std::memcpy(output, input, item_count * d_item_size);
              // Tell the scheduler that items were processed
              consume(in_index, item_count);
              produce(out_index, item_count);
          }
      }

      // Existing connections have been processed. Look for inputs that still
      // need to be handled
      for (int in_index = 0; in_index < ninput_items.size(); in_index++) {
          if (nitems_read(in_index) != ninput_items.at(in_index)) {
              // This input has new samples that have not been processed
              // Look for an available decoder
              bool decoder_found = false;
              for (int out_index = 0; out_index < d_decoders.size(); out_index++) {
                  decoder_info& decoder = d_decoders[out_index];
                  if (decoder.d_input == decoder_info::NO_INPUT) {
                      // Found an available decoder for this input. Connect it
                      // and copy stream items
                      decoder_found = true;
                      decoder.d_input = in_index;

                      // Calculate the number of items to process
                      const int item_count = std::min(ninput_items.at(in_index), noutput_items);

                      const void* input = input_items.at(in_index);
                      void* output = output_items.at(out_index);

                      std::memcpy(output, input, item_count * d_item_size);
                      // Tell the scheduler that items were processed
                      consume(in_index, item_count);
                      produce(out_index, item_count);
                  }
              }
              if (!decoder_found) {
                  // Nothing to do but indicate a decoder deficit
                  local_decoder_surplus -= 1;
              }
          }
      }

      // Update the atomic decoder surplus value
      d_decoder_surplus = local_decoder_surplus;

      // This special value allows different numbers of output samples for
      // different outputs, specified by calling produce()
      return WORK_CALLED_PRODUCE;
    }

    int
    sample_distributor_impl::decoder_surplus() const
    {
        // Atomic read
        return d_decoder_surplus;
    }

  } /* namespace sparsdr */
} /* namespace gr */
