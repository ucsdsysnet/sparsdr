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
          gr::io_signature::make(bands.size(), bands.size(), sizeof(gr_complex))),
      // Begin fields
      d_reconstruct_path(reconstruct_path),
      d_bands(bands),
      d_pipes(),
      d_temp_dir(),
      d_child(0)
{
    start_subprocess();
}

void reconstruct_impl::start_subprocess()
{
    // Start assembling the command
    std::vector<std::string> arguments;
    // First argument: Program name
    arguments.push_back("sparsdr_reconstruct");
    arguments.push_back("--no-progress-bar");
    // Debug log output
    arguments.push_back("--log-level");
    arguments.push_back("INFO");

    // Create a temporary directory for the pipes
    std::string temp_dir("sparsdr_reconstruct_XXXXXX");
    const auto mkdtemp_status = ::mkdtemp(&temp_dir.front());
    if (mkdtemp_status == nullptr) {
        std::cerr << "sparsdr::reconstruct failed to create temporary \
                directory: "
                  << ::strerror(errno) << '\n';
        return;
    }
    d_temp_dir = temp_dir;
    // Create a pipe for compressed things
    const auto compressed_pipe = d_temp_dir + "/compressed.pipe";
    const auto compressed_pipe_status = ::mkfifo(compressed_pipe.c_str(), 0600);
    if (compressed_pipe_status != 0) {
        std::cerr << "sparsdr::reconstruct failed to create a named pipe: "
                  << ::strerror(errno) << '\n';
        return;
    }
    d_pipes.emplace_back(std::move(compressed_pipe));
    // Add the source argument to the command
    arguments.push_back("--source");
    arguments.push_back(compressed_pipe);

    // Create a pipe for each band
    for (auto iter = d_bands.begin(); iter != d_bands.end(); ++iter) {
        // Get index for file name
        const auto i = std::distance(d_bands.begin(), iter);
        const auto pipe_path = make_pipe_path(d_temp_dir, i);
        const auto pipe_status = ::mkfifo(pipe_path.c_str(), 0600);
        if (pipe_status != 0) {
            std::cerr << "sparsdr::reconstruct failed to create a named pipe: "
                      << ::strerror(errno) << '\n';
            return;
        }
        d_pipes.emplace_back(std::move(pipe_path));

        // Add this band to the command
        arguments.push_back("--decompress-band");
        std::stringstream arg_stream;
        arg_stream << iter->bins() << ":" << iter->frequency() << ":" << pipe_path;
        arguments.push_back(arg_stream.str());
    }

    // Low-level manual fork and exec

    // Assemble arguments for exec
    std::vector<char*> exec_args;
    for (auto& arg : arguments) {
        exec_args.push_back(&arg.front());
    }
    exec_args.push_back(nullptr);
    char* envp[] = { nullptr };

    const auto pid = ::fork();
    if (pid == -1) {
        std::cerr << "sparsdr_reconstruct failed to fork: " << ::strerror(errno) << '\n';
    } else if (pid == 0) {
        // This is the child

        const auto exec_status =
            ::execve(d_reconstruct_path.c_str(), exec_args.data(), envp);
        if (exec_status == -1) {
            std::cerr << "sparsdr_reconstruct failed to exec " << d_reconstruct_path
                      << ": " << ::strerror(errno) << '\n';
            std::exit(-1);
        }
    } else {
        // Successfully started
        d_child = pid;
    }

    // Now that the reconstruct process has started, open the named pipes
    // here

    // Create a file sink to write the compressed samples
    const auto compressed_file_sink =
        gr::blocks::file_sink::make(sizeof(uint32_t), compressed_pipe.c_str());
    connect(this->to_basic_block(), 0, compressed_file_sink, 0);

    for (auto iter = d_bands.begin(); iter != d_bands.end(); ++iter) {
        // Get index for file name
        const auto i = std::distance(d_bands.begin(), iter);
        const auto pipe_path = make_pipe_path(d_temp_dir, i);
        // Create a file source to read this band
        const auto band_file_source =
            gr::blocks::file_source::make(sizeof(gr_complex), pipe_path.c_str());
        // Connect it to the appropriate output of this block
        connect(band_file_source, 0, this->to_basic_block(), i);
    }
}

/*
 * Our virtual destructor.
 */
reconstruct_impl::~reconstruct_impl()
{
    // Stop reconstruct process
    if (d_child != 0) {
        ::kill(d_child, SIGINT);
        ::waitpid(d_child, nullptr, 0);
    }
    // Clean up pipes
    for (const auto& path : d_pipes) {
        ::unlink(path.c_str());
    }
    // Delete temporary directory
    if (!d_temp_dir.empty()) {
        ::rmdir(d_temp_dir.c_str());
    }
}

} /* namespace sparsdr */
} /* namespace gr */
