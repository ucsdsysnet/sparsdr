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


#ifndef INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_H
#define INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_H

#include <sparsdr/api.h>
#include <gnuradio/block.h>

namespace gr {
  namespace sparsdr {

    /*!
     * \brief Reads samples from many named pipes and distributes them to decoders
     * \ingroup sparsdr
     *
     */
    class SPARSDR_API sample_distributor : virtual public gr::block
    {
     public:
      typedef boost::shared_ptr<sample_distributor> sptr;

      /*!
       * \brief Return a shared_ptr to a new instance of sparsdr::sample_distributor.
       *
       * To avoid accidental use of raw pointers, sparsdr::sample_distributor's
       * constructor is in a private implementation
       * class. sparsdr::sample_distributor::make is the public interface for
       * creating new instances.
       *
       * \param item_size The size of stream items to process
       * \param pipe_paths the paths to one or more named pipes to read samples
       * from
       */
      static sptr make(int item_size, const std::vector<std::string>& pipe_paths);

      /**
       * \return the number of decoders this block has available but did not use
       * in the last call to general_work()
       *
       * This function is safe to call from any thread.
       *
       * A negative value means that not enough decoders are available for the
       * number of active inputs.
       */
      virtual int decoder_surplus() const = 0;

    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_H */
