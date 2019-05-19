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

#include <sparsdr/api.h>
#include <sparsdr/band_spec.h>
#include <gnuradio/hier_block2.h>

namespace gr {
  namespace sparsdr {

    /*!
     * \brief The SparSDR reconstruct block receives compressed samples
     * and reconstructs signals from one or more bands
     * \ingroup sparsdr
     *
     */
    class SPARSDR_API reconstruct : virtual public gr::hier_block2
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
       * \param reconstruct_path the path to the sparsdr_reconstruct executable
       */
      static sptr make(std::vector<::gr::sparsdr::band_spec> bands, const std::string& reconstruct_path = "sparsdr_reconstruct");
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_H */
