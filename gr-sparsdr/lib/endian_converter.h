#include <uhd/convert.hpp>

namespace gr {
namespace sparsdr {

/**
 * A converter that converts 32-bit big-endian samples into 32-bit
 * little-endian samples
 *
 * The standard UHD converter, with a wire format of sc16 and CPU format of
 * sc16, does this conversion:
 *
 *      -----------------
 *      | A | B | C | D |   Input
 *      -----------------
 *        0   1   2   3     Address
 *      -----------------
 *      | B | A | D | C |   Output
 *      -----------------
 *
 * The SparSDR sample format requires a different conversion, which is
 * implemented here:
 *
 *      -----------------
 *      | A | B | C | D |   Input
 *      -----------------
 *        0   1   2   3     Address
 *      -----------------
 *      | D | C | B | A |   Output
 *      -----------------
 *
 * This converts each 4-byte sample from big-endian to little-endian.
 */
class endian_converter : public ::uhd::convert::converter
{
public:
    endian_converter();

    virtual void set_scalar(const double) override;

    /**
     * Registers this converter and the sparsdr_sample_item32_be type
     */
    static void register_converter();

private:
    virtual void
    operator()(const input_type& in, const output_type& out, const size_t num) override;
};

} // namespace sparsdr
} // namespace gr
