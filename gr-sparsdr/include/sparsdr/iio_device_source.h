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


#ifndef INCLUDED_SPARSDR_IIO_DEVICE_SOURCE_H
#define INCLUDED_SPARSDR_IIO_DEVICE_SOURCE_H

#include <gnuradio/sync_block.h>
#include <sparsdr/api.h>
#include <cstddef>

struct iio_device;

namespace gr {
namespace sparsdr {

/*!
 * \brief A source that reads samples from an IIO device
 *
 * This block is similar to the gr-iio iio_device_source (
 * https://github.com/analogdevicesinc/gr-iio/blob/master/lib/device_source_impl.cc ), but
 * it is simpler and works correctly with a SparSDR-mode Pluto device.
 *
 * \ingroup sparsdr
 *
 */
class SPARSDR_API iio_device_source : virtual public gr::sync_block
{
public:
    typedef boost::shared_ptr<iio_device_source> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::iio_device_source.
     *
     * To avoid accidental use of raw pointers, sparsdr::iio_device_source's
     * constructor is in a private implementation
     * class. sparsdr::iio_device_source::make is the public interface for
     * creating new instances.
     *
     * \param device The IIO device to read samples from
     * \param channel The name of the channel on the provided device to read
     * samples from
     * \param buffer_size_samples The number of samples in the buffer used to
     * read from the IIO device and write to the block output buffer
     *
     * This block does not take ownership of the IIO device or the associated
     * context. Other code may need to destroy the IIO context after this
     * block is destroyed.
     */
    static sptr
    make(iio_device* device, const std::string& channel, std::size_t buffer_size_samples);
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_IIO_DEVICE_SOURCE_H */
