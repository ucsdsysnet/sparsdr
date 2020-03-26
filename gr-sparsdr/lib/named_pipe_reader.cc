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

/*
 * Notes on how things work with named pipes:
 *
 * open() in blocking mode blocks until another process opens the pipe in
 * write-only mode.
 *
 * In non-blocking mode:
 *
 * read() returns -1 and sets errno = EAGAIN if the pipe is still open but
 * nothing is available to read.
 *
 * The other process closing its end of the pipe causes an end-of-file condition.
 * When an end-of-file happens:
 * * select() returns immediately
 * * read() returns 0
 */

#include <errno.h>
#include <fcntl.h>
#include <sys/select.h>
#include <system_error>
#include <algorithm>
#include <climits>

#include "named_pipe_reader.h"

namespace gr {
namespace sparsdr {

named_pipe_reader::named_pipe_reader(const std::vector<std::string>& paths) : d_fds()
{
    d_fds.reserve(paths.size());
    for (const std::string& path : paths) {
        const int fd = ::open(path.c_str(), O_CLOEXEC | O_RDONLY);
        if (fd == -1) {
            // Clean up: Close any files that have been opened
            for (int fd : d_fds) {
                ::close(fd);
            }
            throw std::system_error(std::error_code(errno, std::system_category()),
                                    "failed to open a named pipe");
        }

        if (fd >= FD_SETSIZE) {
            throw std::runtime_error("File descriptor is too large to use select()");
        }

        // Now that the pipe is open (indicating that the other end has opened
        // the pipe as well), switch to nonblocking
        int flags = ::fcntl(fd, F_GETFL, 0);
        if (flags == -1) {
            ::close(fd);
            // Close any other open files
            for (int fd : d_fds) {
                ::close(fd);
            }
            throw std::system_error(std::error_code(errno, std::system_category()),
                                    "failed to open a named pipe");
        }
        flags |= O_NONBLOCK;
        const int status = ::fcntl(fd, F_SETFL, flags);
        if (status == -1) {
            ::close(fd);
            // Close any other open files
            for (int fd : d_fds) {
                ::close(fd);
            }
            throw std::system_error(std::error_code(errno, std::system_category()),
                                    "failed to open a named pipe");
        }

        d_fds.push_back(fd);
    }
}

void named_pipe_reader::wait_for_samples()
{
    fd_set read_files;
    FD_ZERO(&read_files);
    fd_set write_files;
    FD_ZERO(&write_files);
    fd_set except_files;
    FD_ZERO(&except_files);

    int max_fd_plus_one = 0;
    for (int fd : d_fds) {
        // Skip pipes that have already closed
        if (fd == -1) {
            continue;
        }
        if (fd + 1 > max_fd_plus_one) {
            max_fd_plus_one = fd + 1;
        }

        FD_SET(fd, &read_files);
        FD_SET(fd, &except_files);
    }

    // If no file descriptors were added, return immediately
    if (max_fd_plus_one == 0) {
        return;
    }

    // An extremely long timeout
    timeval timeout;
    timeout.tv_sec = LONG_MAX;
    timeout.tv_usec = 0;

    ::select(max_fd_plus_one, &read_files, &write_files, &except_files, &timeout);
}

ssize_t named_pipe_reader::read_samples(std::size_t index, void* buffer, size_t bytes)
{
    int& fd = d_fds.at(index);

    if (fd == -1) {
        // Already closed
        return 0;
    }

    // Try to read
    const ssize_t read_count = ::read(fd, buffer, bytes);

    if (read_count == 0) {
        // Pipe is now closed
        ::close(fd);
        fd = -1;
        return 0;
    } else if (read_count == -1) {
        if (errno == EAGAIN) {
            // Nothing available to read, but this is not really a problem.
            return -1;
        } else {
            // Some other error
            throw std::system_error(std::error_code(errno, std::system_category()),
                                    "pipe read problem");
        }
    } else {
        // Successful read
        return read_count;
    }
}

bool named_pipe_reader::all_pipes_closed() const
{
    // all_of returns true if the range is empty
    return std::all_of(d_fds.begin(), d_fds.end(), [](int fd) { return fd == -1; });
}

bool named_pipe_reader::pipe_closed(std::size_t index) { return d_fds.at(index) == -1; }

named_pipe_reader::~named_pipe_reader()
{
    for (int fd : d_fds) {
        if (fd != -1) {
            ::close(fd);
        }
    }
}

} // namespace sparsdr
} // namespace gr
