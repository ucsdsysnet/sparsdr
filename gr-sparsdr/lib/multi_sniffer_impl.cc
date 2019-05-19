#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <stdexcept>

#include <boost/rational.hpp>

#include <gnuradio/io_signature.h>
#include <gnuradio/filter/firdes.h>
#include "multi_sniffer_impl.h"

namespace gr {
  namespace sparsdr {

      namespace {
        /*!
         * \brief Given the interpolation rate, decimation rate and a fractional
         * bandwidth, design a set of taps.
         *
         * \param interpolation interpolation factor (integer > 0)
         * \param decimation decimation factor (integer > 0)
         * \param fractional_bw fractional bandwidth in (0, 0.5)  0.4 works well.
         */
        std::vector<float> design_filter(unsigned int interpolation,
            unsigned int decimation, float fractional_bw);
      }

    multi_sniffer::sptr
    multi_sniffer::make()
    {
      return gnuradio::get_initial_sptr
        (new multi_sniffer_impl());
    }

    /*
     * The private constructor
     */
    multi_sniffer_impl::multi_sniffer_impl()
      : gr::hier_block2("multi_sniffer",
              gr::io_signature::make(0, 0, 0),
              gr::io_signature::make(0, 0, 0)),
        d_sniffers()
    {
    }

    void
    multi_sniffer_impl::add_sniffer(
        const std::string& path,
        gr::basic_block_sptr sniffer)
    {
        add_sniffer(path, sniffer, 0, 0);
    }
    void
    multi_sniffer_impl::add_sniffer(
        const std::string& path,
        gr::basic_block_sptr sniffer,
        uint32_t sample_rate,
        uint32_t sniffer_sample_rate)
    {
        if (d_sniffers.find(path) != d_sniffers.end()) {
            // Already have a sniffer for that path
            return;
        }
        sniffer_blocks new_sniffer_blocks;
        // Create file source
        new_sniffer_blocks.file_source = gr::blocks::file_source::make(
            sizeof(gr_complex),
            path.c_str()
        );
        if (sample_rate != sniffer_sample_rate) {
            // Calculate resampling ratio
            // The rational constructor normalizes the fraction
            boost::rational<uint32_t> resampling(
                sniffer_sample_rate, sample_rate);
            // Create resampler
            new_sniffer_blocks.resampler = gr::filter::rational_resampler_base_ccf::make(
                resampling.numerator(),
                resampling.denominator(),
                design_filter(resampling.numerator(), resampling.denominator(), 0.4)
            );
        }
        new_sniffer_blocks.sniffer = sniffer;

        // Connect blocks
        if (new_sniffer_blocks.resampler) {
            connect(new_sniffer_blocks.file_source, 0, new_sniffer_blocks.resampler, 0);
            connect(new_sniffer_blocks.resampler, 0, new_sniffer_blocks.sniffer, 0);
        } else {
            connect(new_sniffer_blocks.file_source, 0, new_sniffer_blocks.sniffer, 0);
        }

        // Store in map
        d_sniffers.insert(std::make_pair(path, new_sniffer_blocks));
    }

    void
    multi_sniffer_impl::remove_sniffer(const std::string& path)
    {
        const auto found = d_sniffers.find(path);
        if (found != d_sniffers.end()) {
            // found derefs to the path (found->first) and the sniffer_blocks
            // (found->second)
            const sniffer_blocks& found_sniffer_blocks = found->second;
            disconnect(found_sniffer_blocks.file_source);
            if (found_sniffer_blocks.resampler) {
                disconnect(found_sniffer_blocks.resampler);
            }
            disconnect(found_sniffer_blocks.sniffer);
            d_sniffers.erase(found);
        }
    }

    /*
     * Our virtual destructor.
     */
    multi_sniffer_impl::~multi_sniffer_impl()
    {
    }

    namespace {
        // This is translated from the Python version found at
        // https://github.com/gnuradio/gnuradio/blob/267d669eb21c514c18a6ee979f5cf247d251f1ad/gr-filter/python/filter/rational_resampler.py
        std::vector<float>
        design_filter(unsigned int interpolation,
            unsigned int decimation, float fractional_bw)
        {
            if (fractional_bw >= 0.5 || fractional_bw <= 0) {
                throw std::out_of_range("Invalid fractional_bandwidth, must be in (0, 0.5)");
            }
            float beta = 7.0;
            float halfband = 0.5;
            const float rate = static_cast<float>(interpolation)
                / static_cast<float>(decimation);
            float trans_width;
            float mid_transition_band;
            if (rate >= 1.0) {
                trans_width = halfband - fractional_bw;
                mid_transition_band = halfband - trans_width / 2.0;
            } else {
                rate * (halfband - fractional_bw);
                mid_transition_band = rate * halfband - trans_width / 2.0;
            }

            return gr::filter::firdes::low_pass(
                // gain
                interpolation,
                // sampling_freq
                interpolation,
                // cutoff_freq
                mid_transition_band,
                // transition_width
                trans_width,
                // window
                gr::filter::firdes::WIN_KAISER,
                // beta
                beta
            );
        }
    }

  } /* namespace sparsdr */
} /* namespace gr */
