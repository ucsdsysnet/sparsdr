#include "combined_common.h"
#include <algorithm>
#include <cassert>

namespace gr {
namespace sparsdr {
namespace {
/**
 * Returns the lowest frequency of a band
 */
float band_start_frequency(float band_center, std::uint16_t bins, float bandwidth_per_bin)
{
    const float half_bins = float(bins) / 2;
    return band_center - half_bins * bandwidth_per_bin;
}
/**
 * Returns the highest frequency of a band
 */
float band_end_frequency(float band_center, std::uint16_t bins, float bandwidth_per_bin)
{
    const float half_bins = float(bins) / 2;
    return band_center + half_bins * bandwidth_per_bin;
}
} // namespace

/**
 * Chooses a center frequency that allows all the provided bads to be received
 *
 * This function returns true if a center frequency was calculated, or false
 * if no appropriate center frequency exists.
 */
bool choose_center_frequency(const std::vector<gr::sparsdr::band_spec>& bands,
                             float bandwidth,
                             unsigned int fft_size,
                             float* center_frequency)
{
    assert(!std::isnan(bandwidth));
    assert(bandwidth != 0);
    assert(fft_size != 0);
    if (bands.empty()) {
        return false;
    }
    const float bandwidth_per_bin = bandwidth / float(fft_size);
    const auto compare_min_frequency = [bandwidth_per_bin](
                                           const gr::sparsdr::band_spec& band1,
                                           const gr::sparsdr::band_spec& band2) {
        return band_start_frequency(band1.frequency(), band1.bins(), bandwidth_per_bin) <
               band_start_frequency(band2.frequency(), band2.bins(), bandwidth_per_bin);
    };
    const gr::sparsdr::band_spec& min_frequency_band =
        *std::min_element(bands.begin(), bands.end(), compare_min_frequency);
    const auto compare_max_frequency = [bandwidth_per_bin](
                                           const gr::sparsdr::band_spec& band1,
                                           const gr::sparsdr::band_spec& band2) {
        return band_end_frequency(band1.frequency(), band1.bins(), bandwidth_per_bin) <
               band_end_frequency(band2.frequency(), band2.bins(), bandwidth_per_bin);
    };
    const gr::sparsdr::band_spec& max_frequency_band =
        *std::max_element(bands.begin(), bands.end(), compare_max_frequency);

    // Check required bandwidth
    const float min_frequency = band_start_frequency(
        min_frequency_band.frequency(), min_frequency_band.bins(), bandwidth_per_bin);
    const float max_frequency = band_end_frequency(
        max_frequency_band.frequency(), max_frequency_band.bins(), bandwidth_per_bin);
    if ((max_frequency - min_frequency) > bandwidth) {
        return false;
    }
    // Center is halfway between the two extremes
    *center_frequency = (min_frequency + max_frequency) / 2.0f;

    return true;
}

} // namespace sparsdr
} // namespace gr
