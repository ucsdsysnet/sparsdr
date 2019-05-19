/*
 * This application uses a multi_sniffer to read signals
 * from multiple files and decode them as Bluetooth or BLE.
 *
 * Usage: sparsdr_bluetooth_sniffer [path frequency sample_rate]...
 *
 * The path, frequency, and sample rate may be repeated as many times as desired
 * to create multiple Bluetooth decoders to read from multiple files.
 *
 * This application links against the UHD library, although it does not use it.
 * The library prints a version message when it is loaded.
 */

#include <sparsdr/multi_sniffer.h>
#include <gr_bluetooth/multi_sniffer.h>
#include <gnuradio/top_block.h>
#include <boost/lexical_cast.hpp>

static const uint32_t BLUETOOTH_SAMPLE_RATE = 2000000;

int main(int argc, char** argv) {

    const auto sniffer = gr::sparsdr::multi_sniffer::make();

    // Set up sniffers based on arguments
    const int sniffer_count = (argc - 1) / 3;
    for (int i = 0; i < sniffer_count; i++) {
        const char* path = argv[1 + i * 3];
        const char* frequency_str = argv[1 + i * 3 + 1];
        const char* sample_rate_str = argv[1 + i * 3 + 2];

        const double frequency = boost::lexical_cast<double>(frequency_str);
        const uint32_t sample_rate = boost::lexical_cast<uint32_t>(sample_rate_str);

        // Create Bluetooth sniffer
        const auto bluetooth_sniffer = gr::bluetooth::multi_sniffer::make(
            BLUETOOTH_SAMPLE_RATE,
            frequency,
            // Squelch threshold
            10.0,
            // tun
            false
        );

        sniffer->add_sniffer(path, bluetooth_sniffer, sample_rate, BLUETOOTH_SAMPLE_RATE);
    }

    const auto top_block = gr::make_top_block("sparsdr_bluetooth_sniffer");
    top_block->connect(sniffer);
    top_block->run();
    return 0;
}
