#ifndef COMBINED_COMMON_H
#define COMBINED_COMMON_H
#include <sparsdr/band_spec.h>
#include <vector>

namespace gr {
namespace sparsdr {


/**
 * Chooses a center frequency that allows all the provided bads to be received
 *
 * This function returns true if a center frequency was calculated, or false
 * if no appropriate center frequency exists.
 */
bool choose_center_frequency(const std::vector<gr::sparsdr::band_spec>& bands,
                             float bandwidth,
                             unsigned int fft_size,
                             float* center_frequency);

} // namespace sparsdr
} // namespace gr

#endif
