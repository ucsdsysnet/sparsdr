/*
 * This is a simple test program that links to the sparsdr_reconstruct library
 * and reconstructs a few samples.
 *
 * It should print out 1536 complex numbers.
 */


#include <sparsdr_reconstruct.hpp>
#include <iostream>
#include <array>

namespace {

// This function may be called from any thread
void handle_output(void*, const std::complex<float> *samples, size_t num_samples) {
    for (size_t i = 0; i < num_samples; i++) {
        std::cout << samples[i] << '\n';
    }
}

}

int main() {
    using namespace sparsdr;

    sparsdr_reconstruct_band complete_band;
    complete_band.frequency_offset = 0.0f;
    complete_band.bins = 1024;

    sparsdr_reconstruct_config* config = sparsdr_reconstruct_config_init(handle_output, nullptr);
    config->format = SPARSDR_RECONSTRUCT_FORMAT_V2;
    config->compression_fft_size = 1024;
    config->compressed_bandwidth = 100e6f;
    config->bands = &complete_band;
    config->bands_length = 1;

    // Start
    sparsdr_reconstruct_context* context;
    auto status = sparsdr_reconstruct_init(&context, config);
    if (status != SPARSDR_RECONSTRUCT_OK) {
        std::cerr << "sparsdr_reconstruct_init returned " << status << '\n';
        sparsdr_reconstruct_config_free(config);
        return -1;
    }

    // Write some samples
    std::array<uint32_t, 12> samples {
        0x00000000,
        0x80000025,
        0x00000000,
        0x000a0014,
        0x00000000,
        0x80000026,
        0x00000000,
        0x03a4d93f,
        0x00000000,
        0xc0000027,
        0x00989683,
        0x00000000,
    };

    for (uint32_t sample : samples) {
        status = sparsdr_reconstruct_handle_samples(context, &sample, sizeof sample);
        if (status != SPARSDR_RECONSTRUCT_OK) {
            std::cerr << "sparsdr_reconstruct_handle_samples returned " << status << '\n';
            sparsdr_reconstruct_free(context);
            sparsdr_reconstruct_config_free(config);
            return -1;
        }
    }

    // Clean up
    sparsdr_reconstruct_free(context);
    sparsdr_reconstruct_config_free(config);

    return 0;
}
