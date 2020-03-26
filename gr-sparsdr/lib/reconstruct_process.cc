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

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "reconstruct_process.h"

#include <system_error>
#include <sstream>

#include <errno.h>
// paths.h - does this exist beyond Linux?
#include <paths.h>
#include <signal.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>


namespace gr {
namespace sparsdr {

namespace {

/**
 * Creates a temporary directory and returns its path
 */
std::string make_temp_directory()
{
    // https://stackoverflow.com/questions/31068/how-do-i-find-the-temp-directory-in-linux
    // ? The whole issetugid() / secure_getenv() compatibility thing is too difficult.
    // Just use _PATH_TMP.

    std::string directory = _PATH_TMP "/sparsdr_reconstruct_XXXXXX";
    const auto mkdtemp_status = ::mkdtemp(&directory.front());
    if (mkdtemp_status == nullptr) {
        throw std::system_error(std::error_code(errno, std::system_category()),
                                "failed to create temporary directory");
    }
    return directory;
}

std::string make_pipe_path(const std::string& base, const std::string& file_name)
{
    std::stringstream stream;
    stream << base << '/' << file_name;
    return stream.str();
}

void make_named_pipe(const std::string& path)
{
    const auto status = ::mkfifo(path.c_str(), 0600);
    if (status != 0) {
        std::stringstream message_stream;
        message_stream << "failed to create named pipe " << path;
        throw std::system_error(std::error_code(errno, std::system_category()),
                                message_stream.str());
    }
}

} // namespace

reconstruct_process::reconstruct_process(const std::string& executable,
                                         const std::string& input_path,
                                         const std::vector<band_spec>& bands)
{
    if (bands.empty()) {
        throw std::runtime_error("At least one band to reconstruct must be specified");
    }

    // Create temporary directory and find names for the named pipes
    d_temp_dir = make_temp_directory();

    d_pipe_paths.input = make_pipe_path(d_temp_dir, "compressed.pipe");
    d_pipe_paths.outputs.reserve(bands.size());
    for (std::size_t i = 0; i < bands.size(); i++) {
        std::stringstream file_name_stream;
        file_name_stream << i << ".pipe";
        d_pipe_paths.outputs.push_back(
            make_pipe_path(d_temp_dir, file_name_stream.str()));
    }

    // Create named pipes
    make_named_pipe(d_pipe_paths.input);
    for (const std::string& path : d_pipe_paths.outputs) {
        make_named_pipe(path);
    }

    // Set up arguments
    std::vector<std::string> arguments;
    arguments.push_back("sparsdr_reconstruct");
    arguments.push_back("--no-progress-bar");
    arguments.push_back("--log-level");
    arguments.push_back("WARN");
    arguments.push_back("--source");
    arguments.push_back(d_pipe_paths.input);
    for (std::size_t i = 0; i < bands.size(); i++) {
        const std::string& path = d_pipe_paths.outputs.at(i);
        const band_spec& band = bands.at(i);

        arguments.push_back("--decompress-band");
        std::stringstream arg_stream;
        arg_stream << band.bins() << ":" << band.frequency() << ":" << path;
        arguments.push_back(arg_stream.str());
    }

    // Convert arguments into char*s
    std::vector<char*> arg_pointers;
    arg_pointers.reserve(arguments.size());
    for (std::string& argument : arguments) {
        arg_pointers.push_back(&argument.front());
    }
    arg_pointers.push_back(nullptr);
    char* envp[] = { nullptr };

    // Low-level manual fork and exec
    const auto pid = ::fork();
    if (pid == -1) {
        throw std::system_error(std::error_code(errno, std::system_category()),
                                "failed to fork");
    } else if (pid == 0) {
        // This is the child
        const auto exec_status = ::execve(executable.c_str(), arg_pointers.data(), envp);
        if (exec_status == -1) {
            throw std::system_error(std::error_code(errno, std::system_category()),
                                    "failed to exec sparsdr_reconstruct");
        }
    } else {
        // Successfully started, this is the parent
        d_child = pid;
    }
}

reconstruct_process::pipe_paths reconstruct_process::get_pipe_paths()
{
    return d_pipe_paths;
}

reconstruct_process::~reconstruct_process()
{
    // Stop reconstruct process
    ::kill(d_child, SIGINT);
    ::waitpid(d_child, nullptr, 0);
    // Clean up pipes
    ::unlink(d_pipe_paths.input.c_str());
    for (const auto& path : d_pipe_paths.outputs) {
        ::unlink(path.c_str());
    }
    // Delete temporary directory
    ::rmdir(d_temp_dir.c_str());
}

} // namespace sparsdr
} // namespace gr
