/* -*- c++ -*- */
/*
 * Copyright 2020 The Regents of the University of California.
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

#ifndef INCLUDED_SPARSDR_RECONSTRUCT_PROCESS_H
#define INCLUDED_SPARSDR_RECONSTRUCT_PROCESS_H

#include <cstdint>
#include <string>
#include <vector>

#include <sparsdr/band_spec.h>

namespace gr {
namespace sparsdr {

/**
 * Creates named pipes and runs sparsdr_reconstruct
 */
class reconstruct_process
{
public:
    struct pipe_paths {
        std::string input;
        std::vector<std::string> outputs;
    };

    reconstruct_process(const std::string& executable,
                        const std::string& input_path,
                        const std::vector<band_spec>& bands);

    pipe_paths get_pipe_paths();

    ~reconstruct_process();

private:
    /*! \brief The sparsdr_reconstruct child process */
    pid_t d_child;

    /* \brief The paths to the named pipes created in the constructor */
    pipe_paths d_pipe_paths;

    /* \brief The path to the temporary directory containing the named pipes */
    std::string d_temp_dir;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_RECONSTRUCT_IMPL_H */
