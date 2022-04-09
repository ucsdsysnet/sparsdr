/* -*- c++ -*- */
/*
 * Copyright 2022 The Regents of the University of California.
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

#ifndef INCLUDED_SPARSDR_COMPRESSING_SOURCE_H
#define INCLUDED_SPARSDR_COMPRESSING_SOURCE_H

#include <sparsdr/api.h>
#include <cstdint>
#include <string>

namespace gr {
namespace sparsdr {

namespace {
// Forward-declare this class that is used only in compressing_source.cc
struct bin_range;
} // namespace

/**
 * This is a base class for a device that runs SparSDR compression and allows
 * the compression settings to be configured.
 */
class SPARSDR_API compressing_source
{
public:
    /*!
     * \brief Enables or disables compression
     *
     * When compression is disabled, the radio will send uncompressed samples
     * as if it were using a standard FPGA image.
     *
     * Caution: With some images, this setting has no effect and the radio
     * always sends compressed samples.
     */
    virtual void set_compression_enabled(bool enabled) = 0;
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
     * Enables average samples, enables FFT samples,
     * and starts the FFT
     */
    virtual void start_all();
    /**
     * Stops the FFT, disables FFT samples, and disables average samples
     *
     * A stop_all() followed by start_all() can be used to recover from overflow.
     */
    virtual void stop_all();

    /**
     * Sets the size of the FFT used for compression
     *
     * This function should only be called when the FFT is not running
     * (see the set_run_fft() function).
     *
     * The size must be a power of two between 8 and 1024 inclusive.
     *
     * Caution: Changing the FFT size does not change the window values that
     * the time-domain samples are multiplied by. If the FFT size is set to
     * a non-default value, the window must also be changed to something
     * appropriate for the new FFT size. The load_rounded_hann_window function
     * is an easy way to do this.
     */
    virtual void set_fft_size(std::uint32_t size) = 0;

    /**
     * Returns the current configured FFT size
     */
    virtual std::uint32_t fft_size() const = 0;

    /**
     * Sets the shift amount used in the FFT
     *
     * Valid values are in the range [0, 8]. Smaller values increase the
     * probability of numerical overflow in the FFT, but allow more precision
     * with weak signals.
     */
    virtual void set_shift_amount(std::uint8_t scaling) = 0;
    /**
     * Sets the signal level threshold for one bin
     */
    virtual void set_bin_threshold(std::uint16_t bin_index, std::uint32_t threshold) = 0;
    /**
     * Reads bin thresholds from a file at the specified path
     * and applies them
     *
     * Caution: This does not set the gain or shift amount.
     */
    virtual void set_thresholds_from_file(const std::string& path);

    /**
     * Sets the window value for a bin
     *
     * By default, the FPGA applies a Hann window to the time-domain samples.
     * If this function is used to set a different value for each bin,
     * a different window can be used.
     *
     * This function should only be called when the FFT is not running
     * (see the set_run_fft() function).
     */
    virtual void set_bin_window_value(std::uint16_t bin_index, std::uint16_t value) = 0;

    /**
     * Generates a Hann window with rounded integer values for the provided
     * number of bins, and stores the values in the FPGA
     */
    virtual void load_rounded_hann_window(std::uint32_t bins);

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
    virtual void set_bin_spec(const std::string& spec);

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

    virtual ~compressing_source() = default;

private:
    /**
     * Unmasks bins in the provided range and sets the specified threshold
     */
    void apply_bin_range(const bin_range& range);
};

} // namespace sparsdr
} // namespace gr

#endif
