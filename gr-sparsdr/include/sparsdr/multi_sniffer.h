#ifndef INCLUDED_SPARSDR_MULTI_SNIFFER_H
#define INCLUDED_SPARSDR_MULTI_SNIFFER_H

#include <gnuradio/hier_block2.h>
#include <sparsdr/api.h>

namespace gr {
namespace sparsdr {

/*!
 * \brief A hierachical block that can be configured with many
 * sniffer blocks, each reading from a separate file (which may be a named
 * pipe)
 * \ingroup sparsdr
 *
 * This block has no inputs or outputs. By default, it does nothing.
 * add_sniffer() can be called to add a sniffer.
 */
class SPARSDR_API multi_sniffer : virtual public gr::hier_block2
{
public:
    typedef boost::shared_ptr<multi_sniffer> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::multi_sniffer.
     *
     * To avoid accidental use of raw pointers, sparsdr::multi_sniffer's
     * constructor is in a private implementation
     * class. sparsdr::multi_sniffer::make is the public interface for
     * creating new instances.
     */
    static sptr make();

    /*!
     * \brief Adds a sniffer that reads samples from a file
     *
     * This overload of add_sniffer does not create a resampler. The sniffer
     * will be connected directly to the file, and it should work with the
     * sample rate found in the file.
     *
     * The sniffer block must work with one gr_complex input and no outputs.
     *
     * If this block already contains a sniffer reading from the same path,
     * this function has no effect.
     *
     * Because this function modifies the flow graph, it should not be called
     * when the flow graph is running.
     *
     * \param path the path to a file to read from
     * \param sniffer the sniffer block to set up
     */
    virtual void add_sniffer(const std::string& path, gr::basic_block_sptr sniffer) = 0;

    /*!
     * \brief Adds a sniffer that reads samples from a file, with resampling
     *
     * This overload of add_sniffer creates a resampler that converts
     * from sample_rate (as read from the file) to sniffer_sample_rate
     * (as sent to the sniffer). The sniffer should work at
     * sniffer_sample_rate.
     *
     * The sniffer block must work with one gr_complex input and no outputs.
     *
     * If this block already contains a sniffer reading from the same path,
     * this function has no effect.
     *
     * Because this function modifies the flow graph, it should not be called
     * when the flow graph is running.
     *
     * \param path the path to a file to read from
     * \param sniffer the sniffer block to set up
     * \param sample_rate the sample rate of the file, samples/second
     * \param sniffer_sample_rate the sample rate that the sniffer expects,
     * samples/second
     */
    virtual void add_sniffer(const std::string& path,
                             gr::basic_block_sptr sniffer,
                             uint32_t sample_rate,
                             uint32_t sniffer_sample_rate) = 0;

    /*!
     * \brief Removes a sniffer and associated blocks
     *
     * Because this function modifies the flow graph, it should not be called
     * when the flow graph is running.
     *
     * If this block does not contain any sniffer reading from the provided
     * path, this function has no effect.
     *
     * \param path the file path supplied to add_sniffer
     */
    virtual void remove_sniffer(const std::string& path) = 0;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_MULTI_SNIFFER_H */
