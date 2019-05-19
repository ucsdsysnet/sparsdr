#ifndef INCLUDED_SPARSDR_MULTI_SNIFFER_IMPL_H
#define INCLUDED_SPARSDR_MULTI_SNIFFER_IMPL_H

#include <sparsdr/multi_sniffer.h>

#include <map>

#include <gnuradio/filter/rational_resampler_base_ccf.h>
#include <gnuradio/blocks/file_source.h>

namespace gr {
  namespace sparsdr {

    class multi_sniffer_impl : public multi_sniffer
    {
    private:

        /*! \brief The blocks used for one sniffer */
        struct sniffer_blocks {
            /*! \brief The file source */
            gr::blocks::file_source::sptr file_source;
            /*! \brief The resampler (may be null) */
            gr::filter::rational_resampler_base_ccf::sptr resampler;
            /*! \brief The sniffer */
            gr::basic_block_sptr sniffer;
        };

        /*! \brief A map from input file paths to sniffer blocks */
        std::map<std::string, sniffer_blocks> d_sniffers;

     public:
      multi_sniffer_impl();
      ~multi_sniffer_impl();

      virtual void add_sniffer(
          const std::string& path,
          gr::basic_block_sptr sniffer);
      virtual void add_sniffer(
          const std::string& path,
          gr::basic_block_sptr sniffer,
          uint32_t sample_rate,
          uint32_t sniffer_sample_rate);
      virtual void remove_sniffer(const std::string& path);
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_MULTI_SNIFFER_IMPL_H */
