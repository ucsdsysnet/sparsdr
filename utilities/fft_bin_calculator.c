#include <stdlib.h>
#include <stdio.h>
#include <math.h>

// For Pluto fft_size is 1024, capture_bw is 61.44, filter_bw is 56
int   fft_size   = 2048;
float capture_bw = 100;
float filter_bw  = 100; // set to 40 for WBX

int   rnd_digits = 4;

float round_float (float val, int rnd_digits){
    return (round(val * pow(10, rnd_digits)) / pow(10, rnd_digits));
}

void bins_calc (float capture_center_freq, float band_center_freq, float band_bandwidth){

	  int   half_fft, left_bin, right_bin;
    float bin_width, capture_left, filter_left, filter_right,
		      band_left, band_right, left_freq, right_freq;

    // Compute frequency ranges
    bin_width    = capture_bw / fft_size;
    capture_left = capture_center_freq - (capture_bw     /2);
    filter_left  = capture_center_freq - (filter_bw      /2);
    filter_right = capture_center_freq + (filter_bw      /2);
    band_left    = band_center_freq    - (band_bandwidth /2);
    band_right   = band_center_freq    + (band_bandwidth /2);

    // Frequency range check
	  if ((band_left<filter_left) || (band_right>filter_right)){
        printf ("Band frequency out of filter range.\n");
        return;
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
    left_freq  = round_float(capture_left + left_bin     *bin_width, rnd_digits);
    right_freq = round_float(capture_left + (right_bin+1)*bin_width, rnd_digits);
    printf ("Frequency range: %.*f to %.*f\n", rnd_digits, left_freq, rnd_digits, right_freq);

    // FFT half window shift, so center frequency is at bin 0
	  half_fft  = fft_size/2;
    left_bin  = left_bin ^ half_fft;
    right_bin = right_bin ^ half_fft;

    // check if the range is continuous or not
    if (((left_bin < half_fft    ) && (right_bin < half_fft    )) ||
	      ((left_bin > (half_fft-1)) && (right_bin > (half_fft-1))))
        printf ("%d %d\n", left_bin, right_bin);
    else if ((left_bin == half_fft) && (right_bin == (half_fft-1)))
        printf ("0 %d\n", fft_size-1);
    else
        printf ("0 %d and %d %d\n", right_bin, left_bin, fft_size-1);
}

int main( int argc, char *argv[] ) {
	  if (argc!=4)
        printf("Missing arguments: <capture center freq> <band center freq> <band bandwidth>\n");
	  else
        bins_calc(atof(argv[1]), atof(argv[2]), atof(argv[3]));

	  return 0;
}
