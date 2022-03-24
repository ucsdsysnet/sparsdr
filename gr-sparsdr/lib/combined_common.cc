#include "combined_common.h"
#include "fft_bin_calculator.h"
#include <algorithm>
#include <cassert>
#include <iostream>
#include <sstream>

namespace gr {
namespace sparsdr {

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
