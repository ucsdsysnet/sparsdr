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

#ifndef INCLUDED_SPARSDR_RECONSTRUCT_SINK_H
#define INCLUDED_SPARSDR_RECONSTRUCT_SINK_H

#include <gnuradio/sync_block.h>
#include <sparsdr/api.h>
#include <sparsdr_reconstruct.hpp>

namespace gr {
namespace sparsdr {

/*!
 * \brief This simple sink block runs the first part of the reconstruction.
 *
 * This block is not part of the public API.
 *
 * \ingroup sparsdr
 *
 */
class SPARSDR_API reconstruct_sink : virtual public gr::sync_block
{
public:
    typedef boost::shared_ptr<reconstruct_sink> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::reconstruct_sink.
     *
     * To avoid accidental use of raw pointers, sparsdr::reconstruct_sink's
     * constructor is in a private implementation
     * class. sparsdr::reconstruct_sink::make is the public interface for
     * creating new instances.
     */
    static sptr make(::sparsdr::sparsdr_reconstruct_context* context);
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_SINK_H */
