#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <gnuradio/io_signature.h>
#include "average_detector_impl.h"

namespace gr {
  namespace sparsdr {

    average_detector::sptr
    average_detector::make()
    {
      return gnuradio::get_initial_sptr
        (new average_detector_impl());
    }

    /*
     * The private constructor
     */
    average_detector_impl::average_detector_impl()
      : gr::sync_block("average_detector",
              // Each compressed sample is really 8 bytes, but this also works.
              // The work function can reassemble each sample from two 4-byte
              // integers.
              gr::io_signature::make(1, 1, sizeof(uint32_t)),
              gr::io_signature::make(0, 0, 0)),
        d_last_average(),
        d_last_average_mutex()
    {}

    /*
     * Our virtual destructor.
     */
    average_detector_impl::~average_detector_impl()
    {
    }

    int
    average_detector_impl::work(int noutput_items,
        gr_vector_const_void_star &input_items,
        gr_vector_void_star &output_items)
    {
      const uint32_t* in = reinterpret_cast<const uint32_t*>(input_items[0]);
      const int sample_count = noutput_items / 2;
      for (int i = 0; i < sample_count; i++) {
          // Get the first half of the sample and check bit 15, which
          // indicates an average
          const uint32_t sample0 = in[i * 2];
          const bool is_average = (sample0 >> 15) & 1 == 1;
          if (is_average) {
              const time_point now = std::chrono::high_resolution_clock::now();
              std::lock_guard<std::mutex> guard(d_last_average_mutex);
              d_last_average = now;
          }
      }

      // Tell runtime system how many output items we produced.
      return noutput_items;
    }

    std::chrono::high_resolution_clock::time_point
    average_detector_impl::last_average()
    {
        std::lock_guard<std::mutex> guard(d_last_average_mutex);
        return d_last_average;
    }

  } /* namespace sparsdr */
} /* namespace gr */
