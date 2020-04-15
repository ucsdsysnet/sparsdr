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

#include "reconstruct_impl.h"
#include <gnuradio/blocks/file_sink.h>
#include <gnuradio/blocks/file_source.h>
#include <gnuradio/io_signature.h>
#include <signal.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <cstdlib>
#include <iostream>
#include <sstream>

namespace gr {
namespace sparsdr {

namespace {
/*!
 * \brief Creates a name for an output pipe file in temp_dir
 */
std::string make_pipe_path(const std::string& temp_dir, std::size_t index)
{
    std::stringstream stream;
    stream << temp_dir << "/" << index << ".pipe";
    return stream.str();
}
} // namespace

reconstruct::sptr reconstruct::make(std::vector<band_spec> bands,
                                    const std::string& reconstruct_path)
{
    return gnuradio::get_initial_sptr(new reconstruct_impl(bands, reconstruct_path));
}

/*
 * The private constructor
 */
reconstruct_impl::reconstruct_impl(const std::vector<band_spec>& bands,
                                   const std::string& reconstruct_path)
    : gr::hier_block2(
          "reconstruct",
          // One input for compressed samples
          gr::io_signature::make(1, 1, sizeof(uint32_t)),
          // One output per band
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex)))
{
}


/*
 * Our virtual destructor.
 */
reconstruct_impl::~reconstruct_impl() {}

} /* namespace sparsdr */
} /* namespace gr */
