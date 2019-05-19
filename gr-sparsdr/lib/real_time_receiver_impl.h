#ifndef INCLUDED_SPARSDR_REAL_TIME_RECEIVER_IMPL_H
#define INCLUDED_SPARSDR_REAL_TIME_RECEIVER_IMPL_H

#include <sparsdr/real_time_receiver.h>
#include <sparsdr/average_detector.h>
#include <sparsdr/compressing_usrp_source.h>

namespace gr {
  namespace sparsdr {

    class real_time_receiver_impl : public real_time_receiver
    {
     private:
      /*! \brief Average detector block */
      average_detector::sptr d_average_detector;
      /*! \brief USRP configuration interface */
      compressing_usrp_source::sptr d_usrp;
      /*! \brief Expected interval between average samples */
      duration d_expected_average_interval;

     public:
      real_time_receiver_impl(compressing_usrp_source::sptr usrp,
          const std::string& output_path,
          uint32_t threshold,
          mask_range mask);
      ~real_time_receiver_impl();

      // Implement virtual functions
      virtual time_point last_average();
      virtual duration expected_average_interval() const;
      virtual void restart_compression();
    };

  } // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_REAL_TIME_RECEIVER_IMPL_H */
