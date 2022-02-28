#ifndef INCLUDED_SPARSDR_WINDOW_H
#define INCLUDED_SPARSDR_WINDOW_H

#include <cstdint>
#include <vector>

namespace gr {
namespace sparsdr {
namespace window {

/**
 * Generates and returns a vector of values with length equal to bins
 * that is zero at the beginning and end and 65535 in the middle, matching
 * a Hann window
 */
std::vector<std::uint16_t> rounded_hann_window(std::size_t bins);

} // namespace window
} // namespace sparsdr
} // namespace gr

#endif
