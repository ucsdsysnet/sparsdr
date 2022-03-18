#include "combined_common.h"
#include "fft_bin_calculator.h"
#include <algorithm>
#include <cassert>
#include <iostream>
#include <sstream>

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

combined_receiver_setup::combined_receiver_setup(
    float center_frequency,
    const std::vector<simple_band_spec>& bands,
    std::uint32_t threshold,
    const device_properties& device)
{
    // Determine the bins for each requested band
    std::vector<band_spec> reconstruct_bands;
    std::stringstream generated_bin_spec;
    for (std::size_t i = 0; i != bands.size(); i++) {
        const simple_band_spec& requested_band = bands.at(i);
        exact_ranges band_calculated_ranges;
        const int calc_status = bins_calc_hertz(center_frequency,
                                                device.sample_rate,
                                                requested_band.frequency(),
                                                requested_band.bandwidth(),
                                                device.bandwidth,
                                                device.fft_size,
                                                &band_calculated_ranges);

        unsigned int total_bins;

        std::cout << "Band " << i << " (center " << requested_band.frequency()
                  << " Hz, bandwidth " << requested_band.bandwidth() << " Hz): ";
        if (calc_status == 0) {
            std::cout << "Can't determine bins to unmask\n";
            throw std::runtime_error("Can't determine bins to unmask");
        } else if (calc_status == 1) {
            std::cout << "Unmasking bins " << band_calculated_ranges.l_bin1
                      << " (inclusive) to " << band_calculated_ranges.r_bin1
                      << " (exclusive)\n";
            // Append bins to the band specification, with a trailing comma
            generated_bin_spec << band_calculated_ranges.l_bin1 << ".."
                               << band_calculated_ranges.r_bin1 << ":" << threshold
                               << ",";
            total_bins = band_calculated_ranges.r_bin1 - band_calculated_ranges.l_bin1;
        } else {
            std::cout << "Unmasking bins " << band_calculated_ranges.l_bin1
                      << " (inclusive) to " << band_calculated_ranges.r_bin1
                      << " (exclusive) and bins " << band_calculated_ranges.l_bin2
                      << " (inclusive) to " << band_calculated_ranges.r_bin2
                      << " (exclusive)\n";

            // Append two ranges of bins to the band specification, with a trailing comma
            generated_bin_spec << band_calculated_ranges.l_bin1 << ".."
                               << band_calculated_ranges.r_bin1 << ":" << threshold << ","
                               << band_calculated_ranges.l_bin2 << ".."
                               << band_calculated_ranges.r_bin2 << ":" << threshold
                               << ",";

            total_bins = (band_calculated_ranges.r_bin1 - band_calculated_ranges.l_bin1) +
                         (band_calculated_ranges.r_bin2 - band_calculated_ranges.l_bin2);
        }

        // Assemble a bin specification for the inner block
        // This uses absolute frequencies.
        reconstruct_bands.push_back(band_spec(requested_band.frequency(), total_bins));
    }
    // If the bin specification is not empty, remove the trailing comman
    std::string generated_bin_spec_string = generated_bin_spec.str();
    if (!generated_bin_spec_string.empty()) {
        generated_bin_spec_string.pop_back();
    }
    std::cerr << "Generated bin specification: " << generated_bin_spec_string << '\n';

    this->reconstruct_bands = std::move(reconstruct_bands);
    this->generated_bin_spec = std::move(generated_bin_spec_string);
}

} // namespace sparsdr
} // namespace gr
