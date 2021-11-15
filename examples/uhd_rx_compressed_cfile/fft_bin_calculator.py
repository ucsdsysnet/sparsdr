#!/usr/bin/env python3

import sys
from math import ceil, floor

# For Pluto fft_size is 1024, capture_bw is 61.44, filter_bw is 56
fft_size   = 2048
half_fft   = int(fft_size/2)
capture_bw = 100
filter_bw  = 100 # set to 40 for WBX
rnd_digits = 4


def bins_calc (ccf, bcf, bw):
  # Compute frequency ranges
  binw = capture_bw/fft_size
  cl   = ccf - (capture_bw*1.0/2)
  fl   = ccf - (filter_bw*1.0/2)
  fr   = ccf + (filter_bw*1.0/2)
  bl   = bcf - (bw*1.0/2)
  br   = bcf + (bw*1.0/2)

  if (bl<fl) or (br>fr):
    print ("Band frequency out of filter range.")
    return
  # Compute FFT bin number, round to avoid python float errors
  bl = round((bl - cl)/binw, rnd_digits)
  br = round((br - cl)/binw, rnd_digits)

  # find the integer values for bin number
  bl = floor(bl)
  if (ceil(br)==int(br)):
    br = int(br) - 1
  else:
    br = floor(br)

  # FFT half window shift, so center frequency is at bin 0
  bl = bl ^ half_fft
  br = br ^ half_fft

  # check if the range is continuous or not
  if (bl<half_fft and br<half_fft) or (bl>(half_fft-1) and br>(half_fft-1)):
    print (bl, br)
  elif (bl==half_fft and br==(half_fft-1)):
    print (0, fft_size-1)
  else:
    print (0, br, "and", bl, fft_size-1)


if __name__ == "__main__":
  if (len(sys.argv)<4):
    print ("Missing arguments: <capture center freq> <band center freq> <bandwidth>")
  else:
    bins_calc(float(sys.argv[1]), float(sys.argv[2]), float(sys.argv[3]))
