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

#ifndef INCLUDED_SPARSDR_AVERAGE_DETECTOR_IMPL_H
#define INCLUDED_SPARSDR_AVERAGE_DETECTOR_IMPL_H

#include <chrono>
#include <mutex>
#include <sparsdr/average_detector.h>

namespace gr {
  namespace sparsdr {

    class average_detector_impl : public average_detector
    {
     private:
      typedef std::chrono::high_resolution_clock::time_point time_point;

      /*! \brief The time of the last observed average sample */
      time_point d_last_average;
      /*! \brief Mutex that controls access to d_last_average */
      std::mutex d_last_average_mutex;

     public:
      average_detector_impl();
      ~average_detector_impl();

      // Where all the action really happens
      int work(int noutput_items,
         gr_vector_const_void_star &input_items,
         gr_vector_void_star &output_items);

      virtual std::chrono::high_resolution_clock::time_point last_average();
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_AVERAGE_DETECTOR_IMPL_H */
