#include "fft_bin_calculator.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>

static const int rnd_digits = 4;


static float round_float(float val, int rnd_digits)
{
    return (round(val * pow(10, rnd_digits)) / pow(10, rnd_digits));
}

int bins_calc(float capture_center_freq,
              float capture_bw,
              float band_center_freq,
              float band_bandwidth,
              float filter_bw,
              unsigned int fft_size,
              struct exact_ranges* final_ranges)
{
    int half_fft, left_bin, right_bin;
    float bin_width, capture_left, filter_left, filter_right, band_left, band_right,
        left_bin_float, right_bin_float, l_adj, r_adj;

    // Compute frequency ranges
    bin_width = capture_bw / fft_size;
    capture_left = capture_center_freq - (capture_bw / 2);
    filter_left = capture_center_freq - (filter_bw / 2);
    filter_right = capture_center_freq + (filter_bw / 2);
    band_left = band_center_freq - (band_bandwidth / 2);
    band_right = band_center_freq + (band_bandwidth / 2);

    // Frequency range check
    if ((band_left < filter_left) || (band_right > filter_right)) {
        return 0;
    }

    // Compute FFT bin number, round to avoid float errors
    left_bin_float = round_float((band_left - capture_left) / bin_width, rnd_digits) - 2.0;
    right_bin_float = round_float((band_right - capture_left) / bin_width, rnd_digits) + 2.0;

    // find the integer values for bin number
    left_bin = floor(left_bin_float);
    if (ceil(right_bin_float) == floor(right_bin_float))
        right_bin = floor(right_bin_float) - 1;
    else
        right_bin = floor(right_bin_float);

    // Make number of bins an even number for overlapping
    l_adj = round_float(band_left - (capture_left + left_bin * bin_width), rnd_digits);
    r_adj = round_float((capture_left + (right_bin + 1) * bin_width) - band_right,
                        rnd_digits);

    if ((right_bin - left_bin + 1) % 2 != 0) {
        if ((left_bin == 0) || (left_bin == (fft_size / 2)))
            right_bin += 1;
        else if ((right_bin == (fft_size - 1)) || (right_bin == (fft_size / 2) - 1))
            left_bin -= 1;
        else if (r_adj > l_adj)
            right_bin -= 1;
        else // if (r_adj <= l_adj)
            right_bin += 1;
    }

    // Frequency range captured
    final_ranges->l_freq = round_float(capture_left + left_bin * bin_width, rnd_digits);
    final_ranges->r_freq =
        round_float(capture_left + (right_bin + 1) * bin_width, rnd_digits);

    // FFT half window shift, so center frequency is at bin 0
    half_fft = fft_size / 2;
    left_bin = left_bin ^ half_fft;
    right_bin = right_bin ^ half_fft;

    // check if the range is continuous or not
    if (((left_bin < half_fft) && (right_bin < half_fft)) ||
        ((left_bin > (half_fft - 1)) && (right_bin > (half_fft - 1)))) {

        final_ranges->l_bin1 = left_bin;
        final_ranges->r_bin1 = right_bin;
        final_ranges->l_bin2 = 0;
        final_ranges->r_bin2 = 0;
        return 1;

    } else if ((left_bin == half_fft) && (right_bin == (half_fft - 1))) {

        final_ranges->l_bin1 = 0;
        final_ranges->r_bin1 = fft_size - 1;
        final_ranges->l_bin2 = 0;
        final_ranges->r_bin2 = 0;
        return 1;

    } else {

        final_ranges->l_bin1 = 0;
        final_ranges->r_bin1 = right_bin;
        final_ranges->l_bin2 = left_bin;
        final_ranges->r_bin2 = fft_size - 1;
        return 2;
    }
}

static const float HERTZ_PER_MEGAHERTZ = 1e6f;

int bins_calc_hertz(float capture_center_freq,
                    float capture_bw,
                    float band_center_freq,
                    float band_bandwidth,
                    float filter_bw,
                    unsigned int fft_size,
                    struct exact_ranges* final_ranges)
{
    const int status = bins_calc(capture_center_freq / HERTZ_PER_MEGAHERTZ,
                                 capture_bw / HERTZ_PER_MEGAHERTZ,
                                 band_center_freq / HERTZ_PER_MEGAHERTZ,
                                 band_bandwidth / HERTZ_PER_MEGAHERTZ,
                                 filter_bw / HERTZ_PER_MEGAHERTZ,
                                 fft_size,
                                 final_ranges);
    if (status != 0) {
        // Convert the actual frequencies back to hertz
        final_ranges->l_freq *= HERTZ_PER_MEGAHERTZ;
        final_ranges->r_freq *= HERTZ_PER_MEGAHERTZ;
    }
    return status;
}
