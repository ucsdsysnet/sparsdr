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


#ifndef INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_H
#define INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_H

#include <gnuradio/hier_block2.h>
#include <gnuradio/uhd/usrp_source.h>
#include <sparsdr/api.h>

namespace gr {
namespace sparsdr {

/*!
 * \brief A block that wraps a UHD USRP source and allows SparSDR
 * compression settings to be changed
 * \ingroup sparsdr
 *
 */
class SPARSDR_API compressing_usrp_source : virtual public gr::hier_block2
{
public:
    typedef boost::shared_ptr<compressing_usrp_source> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::compressing_usrp_source.
     *
     * To avoid accidental use of raw pointers, sparsdr::compressing_usrp_source's
     * constructor is in a private implementation
     * class. sparsdr::compressing_usrp_source::make is the public interface for
     * creating new instances.
     */
    static sptr make(const ::uhd::device_addr_t& device_addr);

    // Begin general USRP settings

    /*!
     * Set the gain.
     *
     * \param gain the gain in dB
     */
    virtual void set_gain(double gain) = 0;

    /*!
     * Tune to the desired center frequency.
     *
     * \param tune_request the tune request instructions
     * \return a tune result with the actual frequencies
     */
    virtual ::uhd::tune_result_t
    set_center_freq(const ::uhd::tune_request_t tune_request) = 0;

    /*!
     * Set the antenna to use.
     *
     * \param ant the antenna string
     */
    virtual void set_antenna(const std::string& ant) = 0;

    // Begin SparSDR-specific settings

    /*!
     * \brief Enables or disables compression
     *
     * When compression is disabled, the USRP will send uncompressed data
     * as if it were using a standard FPGA image.
     */
    virtual void set_compression_enabled(bool enabled) = 0;

    /*! \brief Enables or disables the FFT for compression */
    virtual void set_fft_enabled(bool enabled) = 0;
    /*! \brief Enables or disables sending of FFT samples */
    virtual void set_fft_send_enabled(bool enabled) = 0;
    /*! \brief Enables or disables sending of average samples */
    virtual void set_average_send_enabled(bool enabled) = 0;

    /*!
     * \brief Enables the FFT and sending of FFT and average samples
     *
     * This is equivalent to calling set_fft_send_enabled(true),
     * set_average_send_enabled(true), and then set_fft_enabled(true).
     */
    virtual void start_all() = 0;

    /*!
     * \brief Disables the FFT and sending of FFT and average samples
     *
     * This is equivalent to calling set_fft_enabled(false),
     * set_average_send_enabled(false), and then
     * set_fft_send_enabled(false).
     */
    virtual void stop_all() = 0;

    /*!
     * \brief Sets the size of the FFT to use when compressing
     *
     * This function should only be called when the FFT is disabled.
     */
    virtual void set_fft_size(uint32_t size) = 0;

    /*!
     * \brief Sets the FFT scaling (what is this?)
     *
     * The default value is 0x6ab.
     *
     * This function should only be called when the FFT is disabled.
     */
    virtual void set_fft_scaling(uint32_t scaling) = 0;

    /*!
     * \brief Sets the threshold for one FFT bin
     *
     * If the magnitude of the signal for a bin is greater than the
     * threshold, the USRP will send a sample with the signal in that bin.
     *
     * @param index the bin number to set the threshold for. This must be
     * less than the FFT size.
     * @param threshold the threshold to set
     */
    virtual void set_threshold(uint16_t index, uint32_t threshold) = 0;

    // set_window_val is not currently implemented. Sam currently
    // doesn't understand what it does.

    /*!
     * \brief Enables or disables the mask for one FFT bin
     *
     * When a bin is masked, the USRP never sends samples from that bin
     * regardless of the signal level. This can be used to ignore
     * frequencies that have constant transmissions.
     *
     * @param index the bin number to set the threshold for. This must be
     * less than the FFT size.
     * @param enabled if the bin should be masked
     */
    virtual void set_mask_enabled(uint16_t index, bool enabled) = 0;

    /*!
     * \brief Sets the weight used to calculate average signal magnitudes
     *
     * After each FFT, the average for each bin is updated using the formula
     * new_average = weight * average + (1 - weight) * new_magnitude
     *
     * Higher weights make the average change more gradually.
     *
     * The weight must be in the range [0, 1].
     *
     * @param weight the weight to set
     */
    virtual void set_average_weight(float weight) = 0;

    /*!
     * \brief Sets the interval between sending of average samples
     *
     * The interval is in units of 10.24 microseconds. After each interval,
     * the USRP will send average samples for all channels.
     *
     * The interval will be rounded up to the nearest power of two.
     * The interval must not be zero.
     */
    virtual void set_average_packet_interval(uint32_t interval) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_COMPRESSING_USRP_SOURCE_H */
