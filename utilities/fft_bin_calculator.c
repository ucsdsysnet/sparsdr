#include <stdlib.h>
#include <stdio.h>
#include <math.h>

const int rnd_digits = 4;

struct exact_ranges {
		float        l_freq;
		float        r_freq;
		unsigned int l_bin1;
		unsigned int r_bin1;
		unsigned int l_bin2;
		unsigned int r_bin2;
};

float round_float (float val, int rnd_digits){
    return (round(val * pow(10, rnd_digits)) / pow(10, rnd_digits));
}

int bins_calc (float capture_center_freq, float capture_bw,
								float band_center_freq, float band_bandwidth,
								float filter_bw, unsigned int fft_size,
								struct exact_ranges * final_ranges){

	  int   half_fft, left_bin, right_bin;
    float bin_width, capture_left, filter_left, filter_right,
		      band_left, band_right;

    // Compute frequency ranges
    bin_width    = capture_bw / fft_size;
    capture_left = capture_center_freq - (capture_bw     /2);
    filter_left  = capture_center_freq - (filter_bw      /2);
    filter_right = capture_center_freq + (filter_bw      /2);
    band_left    = band_center_freq    - (band_bandwidth /2);
    band_right   = band_center_freq    + (band_bandwidth /2);

    // Frequency range check
	  if ((band_left<filter_left) || (band_right>filter_right)){
        return 0;
	  }

    // Compute FFT bin number, round to avoid float errors
    band_left  = round_float((band_left  - capture_left) / bin_width, rnd_digits);
    band_right = round_float((band_right - capture_left) / bin_width, rnd_digits);

    // find the integer values for bin number
    left_bin = floor(band_left);
    if (ceil(band_right)==floor(band_right))
        right_bin = floor(band_right) - 1;
    else
        right_bin = floor(band_right);

    // Frequency range captured
    final_ranges->l_freq = round_float(capture_left + left_bin     *bin_width, rnd_digits);
    final_ranges->r_freq = round_float(capture_left + (right_bin+1)*bin_width, rnd_digits);

    // FFT half window shift, so center frequency is at bin 0
	  half_fft  = fft_size/2;
    left_bin  = left_bin ^ half_fft;
    right_bin = right_bin ^ half_fft;

    // check if the range is continuous or not
    if (((left_bin < half_fft    ) && (right_bin < half_fft    )) ||
	      ((left_bin > (half_fft-1)) && (right_bin > (half_fft-1)))){

				final_ranges-> l_bin1 = left_bin;
				final_ranges-> r_bin1 = right_bin;
				final_ranges-> l_bin2 = 0;
				final_ranges-> r_bin2 = 0;
				return 1;

    } else if ((left_bin == half_fft) && (right_bin == (half_fft-1))) {

				final_ranges-> l_bin1 = 0;
				final_ranges-> r_bin1 = fft_size-1;
				final_ranges-> l_bin2 = 0;
				final_ranges-> r_bin2 = 0;
				return 1;

    } else {

				final_ranges-> l_bin1 = 0;
				final_ranges-> r_bin1 = right_bin;
				final_ranges-> l_bin2 = left_bin;
				final_ranges-> r_bin2 = fft_size-1;
				return 2;

		}
}

int main( int argc, char *argv[] ) {
		struct exact_ranges ranges;
		int    range_count;

	  if (argc!=4) {
        printf("Missing arguments: <capture center freq> <band center freq> <band bandwidth>\n");
	  } else {
				// Pluto:
				// range_count = bins_calc(atof(argv[1]), 61.44, atof(argv[2]), atof(argv[3]), 56.0, 1024, ranges);
				// N210+WBX:
				// range_count = bins_calc(atof(argv[1]), 100.0, atof(argv[2]), atof(argv[3]), 40.0, 2048, ranges);
				// N210+SBX:
				range_count = bins_calc(atof(argv[1]), 100.0, atof(argv[2]), atof(argv[3]), 100.0, 2048, &ranges);

				if (range_count == 0){
						printf ("Band frequency out of filter range.\n");
				} else if (range_count == 1) {
						printf ("Frequency range: %.*f to %.*f\n", rnd_digits, ranges.l_freq, rnd_digits, ranges.r_freq);
						printf ("FFT range      : %d to %d\n", ranges.l_bin1, ranges.r_bin1);
				} else {
						printf ("Frequency range: %.*f to %.*f\n", rnd_digits, ranges.l_freq, rnd_digits, ranges.r_freq);
						printf ("FFT ranges     : %d to %d and %d to %d\n", ranges.l_bin1, ranges.r_bin1, ranges.l_bin2, ranges.r_bin2);
				}
		}

	  return 0;
}
