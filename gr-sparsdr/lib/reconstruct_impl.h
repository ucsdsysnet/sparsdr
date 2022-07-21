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

#ifndef INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H
#define INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H

#include <sparsdr/reconstruct.h>
#include <unistd.h>
#include <boost/noncopyable.hpp>

namespace gr {
namespace sparsdr {

// Inherit from noncopyable to prevent copying d_child
class reconstruct_impl : public reconstruct, public boost::noncopyable
{
private:
public:
    reconstruct_impl(const std::vector<band_spec>& bands,
                     const std::string& sample_format,
                     bool zero_gaps,
                     unsigned int compression_fft_size);
    ~reconstruct_impl();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H */
