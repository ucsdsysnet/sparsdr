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
     * \brief <+description of block+>
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
       */
      static sptr make(int inputs);
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_SAMPLE_DISTRIBUTOR_H */

