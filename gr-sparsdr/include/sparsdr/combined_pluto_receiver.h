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


#ifndef INCLUDED_SPARSDR_COMBINED_PLUTO_RECEIVER_H
#define INCLUDED_SPARSDR_COMBINED_PLUTO_RECEIVER_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>
#include <sparsdr/band_spec.h>

namespace gr {
namespace sparsdr {

/*!
 * \brief A combination of a compressing Pluto source block and a reconstruct
 * block
 * \ingroup sparsdr
 *
 */
class SPARSDR_API combined_pluto_receiver : virtual public gr::hier_block2
{
public:
    typedef boost::shared_ptr<combined_pluto_receiver> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::combined_pluto_receiver.
     *
     * To avoid accidental use of raw pointers, sparsdr::combined_pluto_receiver's
     * constructor is in a private implementation
     * class. sparsdr::combined_pluto_receiver::make is the public interface for
     * creating new instances.
     *
     * \param uri The URI to use when creating the IIO context (this is usually
     * ip:192.168.2.1)
     * \param buffer_size The number of samples in the buffer used to receive
     * compressed samples from the Pluto
     * \param bands a list of bands to reconstruct
     */
    static sptr make(const std::string& uri,
                     std::size_t buffer_size,
                     const std::vector<band_spec>& bands,
                     const std::string& reconstruct_path = "sparsdr_reconstruct");

    // Compressing pluto source delegate functions
    virtual void set_frequency(unsigned long long frequency) = 0;
    virtual void set_gain(double gain) = 0;
    virtual void set_run_fft(bool enable) = 0;
    virtual void set_send_average_samples(bool enable) = 0;
    virtual void set_send_fft_samples(bool enable) = 0;
    virtual void start_all() = 0;
    virtual void stop_all() = 0;
    virtual void set_fft_size(std::uint32_t size) = 0;
    virtual void set_shift_amount(std::uint8_t scaling) = 0;
    virtual void set_bin_threshold(std::uint16_t bin_index, std::uint32_t threshold) = 0;
    virtual void set_bin_window_value(std::uint16_t bin_index, std::uint16_t value) = 0;
    virtual void set_bin_mask(std::uint16_t bin_index) = 0;
    virtual void clear_bin_mask(std::uint16_t bin_index) = 0;
    virtual void set_bin_spec(const std::string& spec) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMBINED_PLUTO_RECEIVER_H */
