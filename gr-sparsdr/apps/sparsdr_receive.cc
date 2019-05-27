/**
 * This application receives compressed samples from a USRP and writes them
 * to a file.
 */

#include <iostream>
#include <chrono>
#include <thread>
#include <signal.h>

#include <boost/program_options.hpp>
#include <boost/lexical_cast.hpp>

#include <gnuradio/top_block.h>
#include <sparsdr/compressing_usrp_source.h>
#include <sparsdr/real_time_receiver.h>

namespace {

sig_atomic_t running = 1;

void shutdown_handler(int) {
    running = 0;
}

void run_receive(const std::string& usrp_address,
        const std::string& antenna,
        const std::string& output_path,
        uint32_t threshold,
        double gain,
        double frequency,
        bool mask_enable,
        uint16_t mask_low,
        uint16_t mask_high);

/*!
 * Parses a bin mask range
 *
 * \param range a string containing the bin range
 * \param enable_mask this will be set to true if the bin range was parsed
 * successfully and is non-empty, or set to false if the bin range is an
 * empty string
 * \param low If the bin range was parsed successfully and is non-empty,
 * this value will be set to the lower bound of the range.
 * \param high If the bin range was parsed successfully and is non-empty,
 * this value will be set to the upper bound of the range.
 *
 * \return true if the bin range is an empty string or was parsed successfully,
 * or false if the bin range could not be parsed
 */
bool parse_mask_bins(const std::string& range, bool* enable_mask, uint16_t* low, uint16_t* high);

}

int main(int argc, char** argv) {
    namespace po = boost::program_options;

    std::string usrp_address;
    std::string antenna;
    std::string output_path;
    uint32_t threshold;
    double gain;
    double frequency;
    std::string mask_bins;

    po::options_description desc("Allowed options");
    desc.add_options()
        ("help", "display help information")
        ("usrp-address", po::value(&usrp_address)->default_value(""),
            "USRP address in the format accepted by the uhd::device_addr_t \
constructor, for example \"addr=192.168.10.2\"")
        ("antenna", po::value(&antenna)->default_value("RX2"),
            "The antenna to receive signals from")
        ("output-path", po::value(&output_path)->default_value("compressed.iqz"),
            "path to the output file to write")
        ("threshold", po::value(&threshold)->default_value(25000),
            "The signal level threshold that determines if samples are sent")
        ("gain", po::value(&gain)->default_value(0.0),
            "The receive gain in decibels")
        ("frequency", po::value(&frequency)->default_value(2.45e9),
            "The center frequency")
        ("mask-bins", po::value(&mask_bins),
            "A range of bins to mask out (disable), formatted as two numbers \
separated by two . characters. The start bin is inclusive, and the \
end bin is exclusive.\nExample: 10..20 masks bins 10 through 19.");

    po::variables_map vm;
    po::store(po::parse_command_line(argc, argv, desc), vm);
    po::notify(vm);

    if (vm.count("help")) {
        std::cout << desc << "\n";
        return 1;
    }

    // Parse mask_bins
    bool mask_enable = false;
    uint16_t mask_low = 0;
    uint16_t mask_high = 0;
    if (!parse_mask_bins(mask_bins, &mask_enable, &mask_low, &mask_high)) {
        std::cerr << "Invalid mask-bins syntax\n";
        return 1;
    }

    run_receive(
        usrp_address,
        antenna,
        output_path,
        threshold,
        gain,
        frequency,
        mask_enable,
        mask_low,
        mask_high);

    return 0;
}

namespace {

void run_receive(const std::string& usrp_address,
        const std::string& antenna,
        const std::string& output_path,
        uint32_t threshold,
        double gain,
        double frequency,
        bool mask_enable,
        uint16_t mask_low,
        uint16_t mask_high) {
    using std::chrono::high_resolution_clock;

    // Clean shutdown in response to SIGINT or SIGHUP
    struct sigaction shutdown_action;
    shutdown_action.sa_handler = shutdown_handler;
    sigaction(SIGINT, &shutdown_action, nullptr);
    sigaction(SIGHUP, &shutdown_action, nullptr);

    const auto address = uhd::device_addr_t(usrp_address);
    const auto usrp = gr::sparsdr::compressing_usrp_source::make(address);

    // Basic USRP configuration
    usrp->set_gain(gain);
    usrp->set_center_freq(frequency);
    usrp->set_antenna("RX2");

    // Set up mask
    gr::sparsdr::mask_range mask;
    if (mask_enable) {
        mask.start = mask_low;
        mask.end = mask_high;
    }

    auto receiver = gr::sparsdr::real_time_receiver::make(usrp, output_path, threshold, mask);
    const auto expected_average_interval = receiver->expected_average_interval();

    auto top_block = gr::make_top_block("real_time_receive");
    top_block->connect(receiver);

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

bool parse_mask_bins(const std::string& range, bool* enable_mask, uint16_t* low, uint16_t* high) {
    if (range.empty()) {
        *enable_mask = false;
        return true;
    } else {
        // Find the ..
        const auto separator_pos = range.find("..");
        if (separator_pos != std::string::npos) {
            // Found, parse low and high values
            const auto low_str = range.substr(0, separator_pos);
            const auto high_str = range.substr(separator_pos + 2);
            try {
                *low = boost::lexical_cast<uint16_t>(low_str);
                *high = boost::lexical_cast<uint16_t>(high_str);
                *enable_mask = true;

                // Sanity check
                if (high < low) {
                    return false;
                }

                return true;
            } catch (boost::bad_lexical_cast&) {
                return false;
            }
        } else {
            // Not found
            return false;
        }
    }
}

}
