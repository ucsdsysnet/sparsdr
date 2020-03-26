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
#ifndef INCLUDED_SPARSDR_NAMED_PIPE_READER_H
#define INCLUDED_SPARSDR_NAMED_PIPE_READER_H

#include <string>
#include <vector>

#include <boost/noncopyable.hpp>

namespace gr {
namespace sparsdr {

/**
 * Reads samples from one or more named pipes, and detects end-of-file
 * conditions
 */
class named_pipe_reader : public boost::noncopyable
{
private:
    /**
     * File descriptors for open named pipes
     *
     * -1 indicates that the writing process has closed its end of the pipe
     * and nothing mroe can be read
     */
    std::vector<int> d_fds;

public:
    named_pipe_reader(const std::vector<std::string>& paths);

    /**
     * Waits until (a) sample(s) is/are avaialble to read from one or more
     * pipes, until an end-of-file occurs on any pipe, or for an unspecified
     * timeout
     */
    void wait_for_samples();

    /**
     * Reads bytes from a pipe
     *
     * If no samples are available to read but the pipe is still open, this
     * function returns -1.
     *
     * If the pipe has been closed, this function returns 0.
     *
     * If an error occurs, this function throws an unspecified exception.
     */
    ssize_t read_samples(std::size_t index, void* buffer, size_t bytes);

    /**
     * Returns the number of pipes this reader is reading
     */
    inline std::size_t size() const { return d_fds.size(); }

    bool all_pipes_closed() const;

    /**
     * Returns true if the pipe at the provided index is known to be closed
     */
    bool pipe_closed(std::size_t index);

    ~named_pipe_reader();
};

} // namespace sparsdr
} // namespace gr

#endif
