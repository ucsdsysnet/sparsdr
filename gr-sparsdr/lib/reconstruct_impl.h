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
    /*! \brief Path to the sparsdr_reconstruct executable */
    std::string d_reconstruct_path;
    /*! \brief The bands to decompress */
    std::vector<band_spec> d_bands;
    /*! \brief Named pipes created that should be cleaned up */
    std::vector<std::string> d_pipes;
    /*!
     * \brief Temporary directory that should be cleaned up, or an empty
     * string if no temporary directory exists
     */
    std::string d_temp_dir;
    /*! \brief The sparsdr_reconstruct child process, or 0 if none exists */
    pid_t d_child;

    void start_subprocess(const std::string& sample_format,
                          bool zero_gaps,
                          unsigned int compression_fft_size);

public:
    reconstruct_impl(const std::vector<band_spec>& bands,
                     const std::string& reconstruct_path,
                     const std::string& sample_format,
                     bool zero_gaps,
                     unsigned int compression_fft_size);
    ~reconstruct_impl();
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H */
