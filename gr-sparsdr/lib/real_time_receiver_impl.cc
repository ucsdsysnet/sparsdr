#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include "real_time_receiver_impl.h"
#include <gnuradio/blocks/file_sink.h>
#include <gnuradio/io_signature.h>

namespace gr {
namespace sparsdr {

real_time_receiver::sptr real_time_receiver::make(compressing_usrp_source::sptr usrp,
                                                  const std::string& output_path,
                                                  uint32_t threshold,
                                                  mask_range mask)
{
    return gnuradio::get_initial_sptr(
        new real_time_receiver_impl(usrp, output_path, threshold, mask));
}

/*
 * The private constructor
 */
real_time_receiver_impl::real_time_receiver_impl(compressing_usrp_source::sptr usrp,
                                                 const std::string& output_path,
                                                 uint32_t threshold,
                                                 mask_range mask)
    : gr::hier_block2("real_time_receiver",
                      gr::io_signature::make(0, 0, 0),
                      gr::io_signature::make(0, 0, 0)),
      d_average_detector(average_detector::make()),
      d_usrp(usrp),
      d_expected_average_interval()
{
    // Configure USRP
    d_usrp->set_compression_enabled(true);
    d_usrp->stop_all();

    // Clear masks and set threshold
    for (uint16_t i = 0; i < 2048; i++) {
        d_usrp->set_mask_enabled(i, false);
        d_usrp->set_threshold(i, threshold);
    }
    // Set masks
    for (uint16_t i = mask.start; i < mask.end; i++) {
        d_usrp->set_mask_enabled(i, true);
    }
    // Mask bins 0, 1, and 2047
    // These have some special properties.
    d_usrp->set_mask_enabled(0, true);
    d_usrp->set_mask_enabled(1, true);
    d_usrp->set_mask_enabled(2047, true);

    // Set average interval
    const uint32_t average_interval = 1 << 14;
    // Average frequency defined in units of 1024 samples at 100 Msps
    // 1 unit = 10.24 microseconds
    d_expected_average_interval =
        std::chrono::nanoseconds(static_cast<uint64_t>(average_interval) * 10240);
    d_usrp->set_average_packet_interval(average_interval);
    // Start compression
    d_usrp->start_all();

    // File output
    auto file_sink = gr::blocks::file_sink::make(4, output_path.c_str());

    // Connect
    connect(d_usrp, 0, d_average_detector, 0);
    connect(d_usrp, 0, file_sink, 0);
}

real_time_receiver::time_point real_time_receiver_impl::last_average()
{
    return d_average_detector->last_average();
}

real_time_receiver::duration real_time_receiver_impl::expected_average_interval() const
{
    return d_expected_average_interval;
}

void real_time_receiver_impl::restart_compression()
{
    d_usrp->stop_all();
    d_usrp->start_all();
}

/*
 * Our virtual destructor.
 */
real_time_receiver_impl::~real_time_receiver_impl()
{
    // Return the USRP to normal non-compressing mode
    d_usrp->stop_all();
    d_usrp->set_compression_enabled(false);
}

} /* namespace sparsdr */
} /* namespace gr */
