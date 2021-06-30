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

#ifndef INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_H
#define INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>
#include <cstdint>

namespace gr {
namespace sparsdr {

/*!
 * \brief Connects to a suitably configured ADALM-Pluto radio and reads
 * compressed samples
 *
 * \ingroup sparsdr
 *
 */
class SPARSDR_API compressing_pluto_source : virtual public gr::hier_block2
{
public:
    typedef boost::shared_ptr<compressing_pluto_source> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of
     * sparsdr::compressing_pluto_source.
     *
     * To avoid accidental use of raw pointers,
     * sparsdr::compressing_pluto_source's constructor is in a private
     * implementation class. sparsdr::compressing_pluto_source::make is the public
     * interface for creating new instances.
     *
     * \param uri The URI to use to create the IIO context. Example value:
     * `ip:192.168.2.1`
     */
    static sptr make(const std::string& uri);

    // Front-end settings

    /**
     * Sets the center frequency to receive
     *
     * \param frequency the center frequency in hertz
     */
    virtual void set_frequency(unsigned long long frequency) = 0;

    /**
     * Sets the receive gain
     *
     * \param gain the gain in decibels
     */
    virtual void set_gain(double gain) = 0;


    // Compression settings

    /**
     * Enables or disables the compression features
     *
     * When compression is disabled, the device acts like a nomal Pluto
     * radio and sends uncompressed samples.
     *
     * When compression is enabled, the device can be configured to send
     * compressed samples.
     */
    virtual void set_enable_compression(bool enable) = 0;

    /**
     * Enables or disables running the FFT and sending the types of samples
     * that are enabled
     */
    virtual void set_run_fft(bool enable) = 0;
    /**
     * Enables or disables the sending of average samples
     */
    virtual void set_send_average_samples(bool enable) = 0;
    /**
     * Enables or disables the sending of FFT samples (also sometimes called
     * data samples)
     */
    virtual void set_send_fft_samples(bool enable) = 0;

    /**
     * Enables compression, enables FFT samples, enables average samples,
     * and starts the FFT
     */
    virtual void start_all() = 0;
    /**
     * Stops the FFT, disables FFT samples, disables average samples, and
     * disables compression
     *
     * A stop_all() followed by start_all() can be used to recover from overflow.
     */
    virtual void stop_all() = 0;

    /**
     * Sets the size of the FFT used for compression
     *
     * This function should only be called when the FFT is not running
     * (see the set_run_fft() function).
     *
     * The size must be a power of two between 8 and 1024 inclusive.
     */
    virtual void set_fft_size(std::uint32_t size) = 0;
    /**
     * Sets the signal level threshold for one bin
     */
    virtual void set_bin_threshold(std::uint16_t bin_index, std::uint32_t threshold) = 0;
    /**
     * Sets the window value for a bin
     *
     * TODO: What is this?
     */
    virtual void set_bin_window_value(std::uint16_t bin_index, std::uint16_t value) = 0;
    /**
     * Enables the mask for a bin, preventing the device from sending samples
     * for a bin even if it is active
     */
    virtual void set_bin_mask(std::uint16_t bin_index) = 0;
    /** Disables the mask for a bin */
    virtual void clear_bin_mask(std::uint16_t bin_index) = 0;

    /**
     * Sets the thresholds and masks for all 1024 bins from a string specification
     *
     * A mask specification contains zero or more threshold groups, separated
     * by commas.
     *
     * A threshold group contains one bin range, a colon `:`, and one threshold
     * value.
     *
     * A bin range can be a single bin number, or two bin numbers separated by
     * two periods `..`. If two numbers are provided, they represent a range
     * of bins. The start of the range is included, and the end of the range
     * is excluded.
     *
     * A threshold value is a non-negative integer.
     *
     * Any bins not specified will be masked (preventing them from sending any
     * samples).
     *
     * Examples
     * <ul>
     * <li>Mask all bins: (empty string)</li>
     * <li>Enable bin 42 with a threshold of 4000: `42:4000`</li>
     * <li>Enable bins 100 (inclusive) to 200 (exclusive) with a threshold
     *   of 800: `100..200:800`</li>
     * <li>Enable bins 1000 and 1020, both with a threshold of 8192:
     *   `1000:8192,1020:8192`</li>
     * </ul>
     */
    virtual void set_bin_spec(const std::string& spec) = 0;

    /**
     * Sets the weight used to calculate the average signal level for each bin
     *
     * The average is `average_weight * previous_average + (1 - average_weight) *
     * new_sample`.
     *
     * The weight value must be greater than or equal to 0 and less than 1.
     */
    virtual void set_average_weight(float weight) = 0;

    /**
     * Sets the interval between average samples
     *
     * After this many FFT samples have been sent, the device will send a set
     * of average samples.
     *
     * The interval must be greater than or equal to 8 and less than or equal
     * to 2147483648. It will be rounded up to the nearest power of two.
     */
    virtual void set_average_interval(std::uint32_t interval) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_PLUTO_SOURCE_H */
