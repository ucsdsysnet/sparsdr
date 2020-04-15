/* -*- c++ -*- */

#define SPARSDR_API

// Support for std::vector<gr::sparsdr::band_spec>
// This must be before other includes to avoid a confusing error:
// http://swig.10945.n7.nabble.com/Problems-with-std-vector-i-td1921.html
%include "std_vector.i"
%{
#include "sparsdr/band_spec.h"
%}
%include "sparsdr/band_spec.h"

// Required to support the bands argument in the reconstruct block make function
namespace std {
    %template(band_spec_vector) vector<::gr::sparsdr::band_spec>;
}

%include "gnuradio.i"			// the common stuff

//load generated python docstrings
%include "sparsdr_swig_doc.i"

%{
#include "sparsdr/average_detector.h"
#include "sparsdr/real_time_receiver.h"
#include "sparsdr/multi_sniffer.h"
#include "sparsdr/reconstruct.h"
#include "sparsdr/reconstruct_from_file.h"
#include "sparsdr/mask_range.h"
#include "sparsdr/compressing_usrp_source.h"
#include "sparsdr/average_waterfall.h"
#include "sparsdr/sample_distributor.h"
#include "sparsdr/tagged_wavfile_sink.h"
#include "sparsdr/compressing_plutosdr_source.h"
using namespace gr::sparsdr;
%}

%include "sparsdr/average_detector.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, average_detector);
%include "sparsdr/real_time_receiver.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, real_time_receiver);
%include "sparsdr/multi_sniffer.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, multi_sniffer);
%include "sparsdr/reconstruct.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, reconstruct);
%include "sparsdr/reconstruct_from_file.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, reconstruct_from_file);
%include "sparsdr/compressing_usrp_source.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, compressing_usrp_source);
%include "sparsdr/average_waterfall.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, average_waterfall);

%include "sparsdr/sample_distributor.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, sample_distributor);
%include "sparsdr/tagged_wavfile_sink.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, tagged_wavfile_sink);
%include "sparsdr/compressing_plutosdr_source.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, compressing_plutosdr_source);
