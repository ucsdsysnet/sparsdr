#ifndef SPARSDR_FFT_BIN_CALCULATOR_H
#define SPARSDR_FFT_BIN_CALCULATOR_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * A range of frequencies and corresponding bins
 *
 * This may contain either one or two contiguous ranges of bins.
 * If there is only one range, l_bin2 and r_bin2 must both be set to zero.
 *
 * The frequency fields may be in hertz or megahertz depending on the function
 * that calculated them.
 */
struct exact_ranges {
    /**
     * Frequency of the beginning of this range
     */
    float l_freq;
    /**
     * Frequency of the end of this range
     */
    float r_freq;
    /**
     * Bin number of the beginning of the first range of bins
     */
    unsigned int l_bin1;
    /**
     * Bin number of the end of the first range of bins
     */
    unsigned int r_bin1;
    /**
     * Bin number of the beginning of the second range of bins,
     * or zero if there is no second range
     */
    unsigned int l_bin2;
    /**
     * Bin number of the end of the second range of bins,
     * or zero if there is no second range
     */
    unsigned int r_bin2;
};

/**
 * Calculates the range (or two ranges) of bins that should be unmasked
 * to correspond to a range of frequencies
 *
 * @param capture_center_freq the center frequency, in megahertz, of the capture
 * @param capture_bw The sample rate, in millions of samples per second, used to
 *   receive the signals (if there were no analog filter, this would be the same
 *   as filter_bw)
 * @param band_center_freq The center frequency, in megahertz, of the desired
 *   band to unmask
 * @param band_bandwidth The bandwidth, in megahertz, of the band to unmask
 * @param filter_bw The effective analog bandwidth, in megahertz, used to
 *   receive the signals.
 *   This must be less than or equal to capture_bw.
 * @param final_ranges a non-null pointer to an exact_ranges struct.
 *   The struct may be uninitialized. If this function returns a value other
 *   than 0, it initializes the fields of this struct with the actual frequency
 *   range and the range(s) of bins to unmask.
 *
 * All frequency arguments are absolute (not relative to capture_center_freq).
 *
 * @return 0 if any of the desired band is outside the available range
 *   (defined by capture_center_freq and filter_bw), 1 if one range of bins
 *   should be unmasked, or 2 if two ranges of bins should be unmasked
 */
int bins_calc(float capture_center_freq,
              float capture_bw,
              float band_center_freq,
              float band_bandwidth,
              float filter_bw,
              unsigned int fft_size,
              struct exact_ranges* final_ranges);

/**
 * Calculates the range (or two ranges) of bins that should be unmasked
 * to correspond to a range of frequencies
 *
 * While bins_calc uses frequencies in megahertz, this function uses frequencies
 * in hertz.
 *
 * @param capture_center_freq the center frequency, in hertz, of the capture
 * @param capture_bw The sample rate, in samples per second, used to receive the
 *   signals (if there were no analog filter, this would be the same as
 *   filter_bw)
 * @param band_center_freq The center frequency, in hertz, of the desired band
 *   to unmask
 * @param band_bandwidth The bandwidth, in hertz, of the band to unmask
 * @param filter_bw The effective analog bandwidth, in hertz, used to receive the
 *   signals.
 *   This must be less than or equal to capture_bw.
 * @param final_ranges a non-null pointer to an exact_ranges struct.
 *   The struct may be uninitialized. If this function returns a value other
 *   than 0, it initializes the fields of this struct with the actual frequency
 *   range and the range(s) of bins to unmask.
 *
 * All frequency arguments are absolute (not relative to capture_center_freq).
 *
 * @return 0 if any of the desired band is outside the available range
 *   (defined by capture_center_freq and filter_bw), 1 if one range of bins
 *   should be unmasked, or 2 if two ranges of bins should be unmasked
 */
int bins_calc_hertz(float capture_center_freq,
                    float capture_bw,
                    float band_center_freq,
                    float band_bandwidth,
                    float filter_bw,
                    unsigned int fft_size,
                    struct exact_ranges* final_ranges);

#ifdef __cplusplus
}
#endif

#endif
