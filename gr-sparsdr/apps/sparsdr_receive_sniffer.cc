/*
 * This application performs two functions:
 * 1. Receiving compressed samples from a USRP and writing them to a file,
 * like sparsdr_real_time_receive
 * 2. Reading uncompressed signals from various files and decoding them as
 * Bluetooth, like sparsdr_bluetooth_sniffer
 *
 * Usage: sparsdr_receive_sniffer compressed-output-path [path frequency sample_rate]...
 *
 * The path, frequency, and sample rate may be repeated as many times as desired
 * to create multiple Bluetooth decoders to read from multiple files.
 */

#include <map>
#include <iostream>
#include <chrono>
#include <thread>
#include <mutex>
#include <signal.h>

#include <gnuradio/top_block.h>
#include <sparsdr/compressing_usrp_source.h>
#include <sparsdr/real_time_receiver.h>
#include <sparsdr/multi_sniffer.h>
#include <gr_bluetooth/multi_sniffer.h>
#include <boost/lexical_cast.hpp>

namespace {
static const uint32_t BLUETOOTH_SAMPLE_RATE = 2000000;

sig_atomic_t running = 1;

void shutdown_handler(int) {
    running = 0;
}

}

int main(int argc, char** argv) {
    using std::chrono::high_resolution_clock;
    if (argc < 2) {
        std::cerr << "Usage: sparsdr_receive_sniffer compressed-output-path [path frequency sample_rate]...\n";
        return -1;
    }

    const char* output_path = argv[1];

    const auto address = uhd::device_addr_t();
    const auto usrp = gr::sparsdr::compressing_usrp_source::make(address);

    // Basic USRP configuration
    usrp->set_gain(30);
    usrp->set_center_freq(2.45e9);
    usrp->set_antenna("RX2");
    const auto threshold = 10000;

    auto receiver = gr::sparsdr::real_time_receiver::make(usrp, output_path, threshold);
    const auto expected_average_interval = receiver->expected_average_interval();

    auto top_block = gr::make_top_block("receive_sniffer");
    top_block->connect(receiver);

    // Open sniffer input files, set up sniffers
    const auto sniffer = gr::sparsdr::multi_sniffer::make();

    // Set up sniffers based on arguments
    const int sniffer_count = (argc - 2) / 3;
    for (int i = 0; i < sniffer_count; i++) {
        const char* path = argv[2 + i * 3];
        const char* frequency_str = argv[2 + i * 3 + 1];
        const char* sample_rate_str = argv[2 + i * 3 + 2];

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
    top_block->connect(sniffer);

    // Clean shutdown in response to SIGINT or SIGHUP
    struct sigaction shutdown_action;
    shutdown_action.sa_handler = shutdown_handler;
    sigaction(SIGINT, &shutdown_action, nullptr);
    sigaction(SIGHUP, &shutdown_action, nullptr);

    // Run
    top_block->start();

    // Check for recent average packets, and restart compression if one has
    // not been seen recently enough.
    // This type of overflow indicates that whatever is handling the compressed
    // samples could not process them quickly enough.
    uint32_t restart_count = 0;
    while (running) {
        std::this_thread::sleep_for(expected_average_interval * 2);
        const auto last_average = receiver->last_average();
        const auto since_last_average = high_resolution_clock::now() - last_average;
        if (since_last_average > expected_average_interval * 2) {
            restart_count += 1;
            std::cerr << "Compression internal overflow, restarting\n";
            receiver->restart_compression();
        }
    }

    top_block->stop();
    top_block->wait();

    std::cerr << "Restarted compression " << restart_count << " times\n";
}
