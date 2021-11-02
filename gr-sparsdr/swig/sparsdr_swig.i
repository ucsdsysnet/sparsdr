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
#include "sparsdr/reconstruct.h"
#include "sparsdr/compressing_usrp_source.h"
#include "sparsdr/compressing_pluto_source.h"
#include "sparsdr/iio_device_source.h"
using namespace gr::sparsdr;
%}

%include "sparsdr/reconstruct.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, reconstruct);
%include "sparsdr/compressing_usrp_source.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, compressing_usrp_source);
%include "sparsdr/compressing_pluto_source.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, compressing_pluto_source);
%include "sparsdr/iio_device_source.h"
GR_SWIG_BLOCK_MAGIC2(sparsdr, iio_device_source);
