#ifndef INCLUDED_SPARSDR_AVERAGE_DETECTOR_H
#define INCLUDED_SPARSDR_AVERAGE_DETECTOR_H

#include <gnuradio/sync_block.h>
#include <sparsdr/api.h>
#include <chrono>

namespace gr {
namespace sparsdr {

/*!
 * \brief Detects average samples in a compressed stream and records
 * the time of the last sample
 * \ingroup sparsdr
 *
 */
class SPARSDR_API average_detector : virtual public gr::sync_block
{
public:
    typedef boost::shared_ptr<average_detector> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::average_detector.
     *
     * To avoid accidental use of raw pointers, sparsdr::average_detector's
     * constructor is in a private implementation
     * class. sparsdr::average_detector::make is the public interface for
     * creating new instances.
     */
    static sptr make();

    /*!
     * \brief Returns the time when the last average sample was observed
     *
     * This function is safe to call from any thread.
     */
    virtual std::chrono::high_resolution_clock::time_point last_average() = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_AVERAGE_DETECTOR_H */
