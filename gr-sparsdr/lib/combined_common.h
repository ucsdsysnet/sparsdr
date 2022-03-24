#ifndef COMBINED_COMMON_H
#define COMBINED_COMMON_H
#include <sparsdr/band_spec.h>
#include <sparsdr/simple_band_spec.h>
#include <string>
#include <vector>

namespace gr {
namespace sparsdr {

// Things used for simple combined receiver blocks

struct device_properties {
    /** Number of bins in the receive FFT */
    unsigned int fft_size;
    /** Receive sample rate, in hertz */
    float sample_rate;
    /** Receive bandwidth, in hertz (this must not be greater than sample_rate) */
    float bandwidth;
};

/**
 * Settings derived from a list of simple_band_specs that can be used to
 * configure the radio and reconstruction software
 */
struct combined_receiver_setup {
    /** Bands used for reconstruction */
    std::vector<band_spec> reconstruct_bands;
    /** Specification of the unmasked bins and thresholds to configure */
    std::string generated_bin_spec;

    /**
     * Attempts to make a receiver setup
     *
     * @param center_frequency the center frequency, in hertz
     * @param bands the bands to receive (all frequencies are absolute)
     * @param threshold the threshold to apply to all unmasked bins
     * @param device information about the radio
     */
    combined_receiver_setup(float center_frequency,
                            const std::vector<simple_band_spec>& bands,
                            std::uint32_t threshold,
                            const device_properties& device);
};

} // namespace sparsdr
} // namespace gr

#endif
