#include "window.h"
#include <sparsdr/compressing_source.h>
#include <boost/lexical_cast.hpp>
#include <stdexcept>
#include "threshold_file.h"

namespace gr {
namespace sparsdr {

namespace {
struct bin_range {
public:
    std::uint16_t start_bin;
    std::uint16_t end_bin;
    std::uint32_t threshold;

    static bin_range parse(const std::string& range_spec, std::uint32_t fft_size)
    {
        const auto colon_index = range_spec.find(":");
        if (colon_index == std::string::npos) {
            throw std::invalid_argument("No : character in range specification");
        }
        const auto before_colon = range_spec.substr(0, colon_index);
        const auto after_colon =
            range_spec.substr(colon_index + 1, range_spec.length() - colon_index - 1);

        // Parse the single number or range before the colon
        std::uint16_t start_bin = 0;
        std::uint16_t end_bin = 0;
        const auto dots_index = before_colon.find("..");
        if (dots_index == std::string::npos) {
            // Just one number
            const std::uint16_t bin = boost::lexical_cast<std::uint16_t>(before_colon);
            start_bin = bin;
            end_bin = bin + 1;
        } else {
            const auto before_dots = before_colon.substr(0, dots_index);
            const auto after_dots = before_colon.substr(
                dots_index + 2, before_colon.length() - dots_index - 2);
            start_bin = boost::lexical_cast<std::uint16_t>(before_dots);
            end_bin = boost::lexical_cast<std::uint16_t>(after_dots);
        }

        if (start_bin >= fft_size || end_bin > fft_size) {
            throw std::invalid_argument("Bin number too large");
        }

        const std::uint32_t threshold = boost::lexical_cast<std::uint32_t>(after_colon);
        return bin_range{ start_bin, end_bin, threshold };
    }
};
} // namespace

void compressing_source::start_all()
{
    set_send_average_samples(true);
    set_send_fft_samples(true);
    set_run_fft(true);
}

void compressing_source::stop_all()
{
    set_run_fft(false);
    set_send_fft_samples(false);
    set_send_average_samples(false);
}

void compressing_source::load_rounded_hann_window(std::uint32_t bins)
{
    const std::vector<std::uint16_t> window = window::rounded_hann_window(bins);
    assert(window.size() == bins);
    for (std::uint16_t bin = 0; bin != bins; bin++) {
        set_bin_window_value(bin, window.at(bin));
    }
}

void compressing_source::set_bin_spec(const std::string& spec)
{
    const std::uint32_t local_fft_size = fft_size();
    // Mask all bins
    for (std::uint16_t bin = 0; bin < local_fft_size; bin++) {
        set_bin_mask(bin);
    }
    // Parse specification
    if (spec.empty()) {
        // Leave all bins masked
        return;
    }
    std::string::size_type start_index = 0;
    while (true) {
        const auto next_comma_index = spec.find(",", start_index);
        if (next_comma_index == std::string::npos) {
            // No more commas. Try to parse the rest of the string, then stop
            const auto last_part = spec.substr(start_index, spec.length() - start_index);
            const auto last_bin_range = bin_range::parse(last_part, local_fft_size);
            apply_bin_range(last_bin_range);
            break;
        } else {
            const auto current_part =
                spec.substr(start_index, next_comma_index - start_index);
            const auto bin_range = bin_range::parse(current_part, local_fft_size);
            apply_bin_range(bin_range);
            // Start searching for the next comma after this one
            start_index = next_comma_index + 1;
        }
    }
}

void compressing_source::apply_bin_range(const bin_range& range)
{
    for (std::uint16_t bin = range.start_bin; bin < range.end_bin; bin++) {
        set_bin_threshold(bin, range.threshold);
        clear_bin_mask(bin);
    }
}

void compressing_source::configure_from_file(const std::string& path)
{
    threshold_file file(path, fft_size());
    // TODO
}

} // namespace sparsdr
} // namespace gr
