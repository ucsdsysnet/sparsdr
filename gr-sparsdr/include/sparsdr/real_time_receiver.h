#ifndef INCLUDED_SPARSDR_REAL_TIME_RECEIVER_H
#define INCLUDED_SPARSDR_REAL_TIME_RECEIVER_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>
#include <sparsdr/compressing_usrp_source.h>
#include <sparsdr/mask_range.h>
#include <chrono>
#include <string>

namespace gr {
namespace sparsdr {

/*!
 * \brief A hierarchical block that receives compressed samples from
 * a USRP and writes them to a file
 * \ingroup sparsdr
 *
 * The file may be a named pipe that can send data to a decompression
 * process for real-time use.
 *
 * This block does not have any inputs or outputs.
 *
 * When a real_time_receiver is destructed it disables compression on
 * its USRP, returning it to normal mode.
 */
class SPARSDR_API real_time_receiver : virtual public gr::hier_block2
{
public:
    typedef boost::shared_ptr<real_time_receiver> sptr;

    /*! \brief The duration type returned by expected_average_interval() */
    typedef std::chrono::nanoseconds duration;
    /*! \brief The time point type returned by last_average() */
    typedef std::chrono::high_resolution_clock::time_point time_point;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::real_time_receiver.
     *
     * To avoid accidental use of raw pointers, sparsdr::real_time_receiver's
     * constructor is in a private implementation
     * class. sparsdr::real_time_receiver::make is the public interface for
     * creating new instances.
     *
     * \param usrp An existing USRP source. The center frequency, antenna,
     * and other application-specific settings should already be configured.
     * The bandwidth should be left at its default value. The USRP sink
     * must be using data type sc16, not the default fc32.
     *
     * \param output_path the path to the file to write compressed samples to.
     * This file may be a named pipe.
     *
     * \param threshold the initial threshold for all bins
     *
     * \param mask an optional range of bins to mask out. The default
     * value does not mask any bins.
     */
    static sptr make(compressing_usrp_source::sptr usrp,
                     const std::string& output_path,
                     uint32_t threshold = 25000,
                     ::gr::sparsdr::mask_range mask = ::gr::sparsdr::mask_range());

    /*!
     * \brief Returns the expected time interval between average samples
     * from the USRP
     */
    virtual duration expected_average_interval() const = 0;

    /*!
     * \brief Returns the time of the last average sample seen from the USRP
     */
    virtual time_point last_average() = 0;

    /*!
     * \brief Disables and re-enables the FFT on the USRP
     *
     * This can be used to start compression after it stops due to an
     * internal overflow.
     */
    virtual void restart_compression() = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_REAL_TIME_RECEIVER_H */
