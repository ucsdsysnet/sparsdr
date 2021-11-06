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

#ifndef INCLUDED_SPARSDR_COMBINED_PLUTO_RECEIVER_IMPL_H
#define INCLUDED_SPARSDR_COMBINED_PLUTO_RECEIVER_IMPL_H

#include <sparsdr/combined_pluto_receiver.h>
#include <sparsdr/compressing_pluto_source.h>
#include <sparsdr/reconstruct.h>

namespace gr {
namespace sparsdr {

class combined_pluto_receiver_impl : public combined_pluto_receiver
{
private:
    // Enclosed blocks
    compressing_pluto_source::sptr d_pluto;
    reconstruct::sptr d_reconstruct;

public:
    combined_pluto_receiver_impl(const std::string& uri,
                                 std::size_t buffer_size,
                                 const std::vector<band_spec>& bands,
                                 const std::string& reconstruct_path);
    ~combined_pluto_receiver_impl();

    // Where all the action really happens
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMBINED_PLUTO_RECEIVER_IMPL_H */
