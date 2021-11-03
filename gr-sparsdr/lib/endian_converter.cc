#include "endian_converter.h"
#include <boost/make_shared.hpp>
#include <cstdint>

namespace gr {
namespace sparsdr {

endian_converter::endian_converter()
{
    // Nothing to initialize
}

void endian_converter::set_scalar(const double)
{
    // Nothing (scaling is not used)
}

void endian_converter::
operator()(const input_type& in, const output_type& out, const size_t num)
{
    // This works correctly only on little-endian targets.
    const std::uint32_t* samples_in = reinterpret_cast<const std::uint32_t*>(in[0]);
    std::uint32_t* samples_out = reinterpret_cast<std::uint32_t*>(out[0]);
    for (std::size_t i = 0; i < num; i++) {
        // Swap all four bits
        const std::uint32_t sample_in = samples_in[i];
        samples_out[i] = (sample_in & 0xff) << 24 | ((sample_in >> 8) & 0xff) << 16 |
                         ((sample_in >> 16) & 0xff) << 8 | ((sample_in >> 24) & 0xff);
    }
}

void endian_converter::register_converter()
{
    uhd::convert::id_type id;
    // The USRP2 driver takes the wire format sc16 and appends _item32_be
    id.input_format = "sc16_item32_be";
    id.num_inputs = 1;
    // The sparsdr_sample format is just a name for a byte stream
    id.output_format = "sparsdr_sample";
    id.num_outputs = 1;

    uhd::convert::register_bytes_per_item("sparsdr_sample", 4);
    uhd::convert::register_converter(id,
                                     []() -> uhd::convert::converter::sptr {
                                         return boost::make_shared<endian_converter>();
                                     },
                                     0 /* Priority */);
}

} // namespace sparsdr
} // namespace gr

// Define an empty destructor for the converter base class
//
// Some builds of the uhd library are apparently missing this.
namespace uhd {
namespace convert {
converter::~converter() {}
} // namespace convert
} // namespace uhd
