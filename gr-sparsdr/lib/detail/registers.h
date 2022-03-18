#ifndef INCLUDED_SPARSDR_PRIVATE_REGISTERS_H
#define INCLUDED_SPARSDR_PRIVATE_REGISTERS_H

namespace gr {
namespace sparsdr {
namespace detail {
namespace registers {
/** Shift amount used for FFT scaling */
constexpr uint8_t SHIFT_AMOUNT = 10;
/** Bin number for threshold adjustment */
constexpr std::uint8_t THRESHOLD_BIN_NUMBER = 11;
/** Per-bin mask set command */
constexpr std::uint8_t MASK = 12;
/** Average weight */
constexpr std::uint8_t AVG_WEIGHT = 13;
/** Average interval */
constexpr std::uint8_t AVG_INTERVAL = 14;
/** Enable FFT sample sending */
constexpr std::uint8_t FFT_SEND = 15;
/** Enable average sample sending */
constexpr std::uint8_t AVG_SEND = 16;
/** Enable FFT */
constexpr std::uint8_t RUN_FFT = 17;
/** Per-bin value for window function */
static const std::uint8_t WINDOW_VAL = 18;
/** Register to enable/disable compression */
constexpr std::uint8_t ENABLE_COMPRESSION = 19;
/** FFT size */
constexpr std::uint8_t FFT_SIZE = 20;
/** Threshold value (used with THRESHOLD_BIN_NUMBER) */
constexpr std::uint8_t THRESHOLD_VALUE = 21;
} // namespace registers
} // namespace detail
} // namespace sparsdr
} // namespace gr

#endif
