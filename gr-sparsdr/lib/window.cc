#include "window.h"
#include <cmath>

namespace gr {
namespace sparsdr {
namespace window {

namespace {
constexpr double PI = 3.14159265358979323846264338327950288;
}

std::vector<std::uint16_t> rounded_hann_window(std::size_t bins)
{
    std::vector<std::uint16_t> values;
    values.reserve(bins);

    for (std::size_t i = 0; i < bins; i++) {
        const double float_value =
            0.5 * (1.0 - std::cos(2.0 * PI * double(i) / double(bins - 1)));
        const std::uint16_t int_value = std::uint16_t(std::round(float_value * 65535.0));
        values.push_back(int_value);
    }

    return values;
}
} // namespace window
} // namespace sparsdr
} // namespace gr
