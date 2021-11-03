/* -*- c++ -*- */
/*
 * Copyright 2021 The Regents of the University of California.
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


#ifndef INCLUDED_SPARSDR_SWAP_16_H
#define INCLUDED_SPARSDR_SWAP_16_H

#include <gnuradio/sync_block.h>
#include <sparsdr/api.h>

namespace gr {
namespace sparsdr {

/*!
 * \brief This block swaps the two 16-bit chunks of each 32-bit sample.
 *
 * This is required when using an N210 and using a method that does not
 * use the "sparsdr_sample" type and register the custom endian converter.
 * The compressing USRP source block does this, so this block should not be
 * used with it.
 *
 * \ingroup sparsdr
 *
 */
class SPARSDR_API swap_16 : virtual public gr::sync_block
{
public:
    typedef boost::shared_ptr<swap_16> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::swap_16.
     *
     * To avoid accidental use of raw pointers, sparsdr::swap_16's
     * constructor is in a private implementation
     * class. sparsdr::swap_16::make is the public interface for
     * creating new instances.
     */
    static sptr make();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_SWAP_16_H */
