#!/usr/bin/env python3

from sys import stdin
from struct import unpack

conv_2_V1   = False
V1_out_file = "v1_out.iqz"

fft_size_log2     = 11
max_fft_size_log2 = 11
FFT_size          = 2**fft_size_log2

fft_time_offset = int(0)
avg_time_offset = int(0)
last_fft_time   = int(0)
last_avg_time   = int(0)

first_zero = 0
after_zero = False
in_FFT     = True
in_avg     = False
after_hdr  = False
FFT_index  = 0

# Clock is 100MHz, and we cut (fft_size_log-1) bits to show start of window
ts = 16.2760417 * (1 << (fft_size_log2-1))

v1_time_bits  = 32-1-max_fft_size_log2
v1_time_mask  = (2**(v1_time_bits))-1
if (conv_2_V1):
  v1_out = open(V1_out_file, "wb")

while True:
  b = stdin.buffer.read(4)
  if (len(b) < 4):
    break

  (value, )    = unpack("I", b)
  (imag, real) = unpack("hh", b)

  is_hdr = (value >> 31) & 0x1
  is_avg = (value >> 30) & 0x1
  time   = (value & 0x3FFFFFFF);
  index  = value
  fft_no = time & 0x1 # simply odd or even time for start of window

  # print ("if hdr:", is_hdr, is_avg, time, "if data:", value)

  if (first_zero!=2):
    if (first_zero==0):
      if (value==0):
        first_zero = 1
      print ("Trying to find a proper header")
      continue
    else: # potential first zero
      if (value==0):
        print ("Trying to find a proper header")
        continue
      elif not is_hdr:
        print ("Trying to find a proper header")
        first_zero = 0 # reset the search
        continue
      else:
        first_zero = 2
        after_zero = True
        fft_time_offset = -time
        avg_time_offset = -time

  if (in_FFT and (value==0)) or (in_avg and (FFT_index==FFT_size) and (value==0)):
    after_zero = True
    print ("(End Frame)")
    continue

  if (after_zero):
    if (value==0):
      first_zero=1
      print ("Error detecting window")
    elif is_hdr:
      if is_avg:
        if (time < last_avg_time):
          avg_time_offset += (1<<30)
        last_avg_time = time
        # Average sample times always have fft_size_log bits tail zero
        fixed_avg_time = ((time & 0x3FFFFFFE) + avg_time_offset) * ts
        print ("Average header at time", fixed_avg_time,"(ns)")
        FFT_index = 0
        in_avg = True
        in_FFT = False
      else:
        if (time < last_fft_time):
          fft_time_offset += (1<<30)
        last_fft_time = time
        fixed_fft_time = (time + fft_time_offset) * ts
        print ("FFT header at time", fixed_fft_time,"(ns)")
        in_avg = False
        in_FFT = True
        after_hdr = True
    else:
      print ("(FFT index)")
      FFT_index = value
    after_zero = False
  else:
    if (in_avg):
      print ("Average, index", FFT_index, ":", value)
      if (conv_2_V1):
        v1_conv = (FFT_index << (v1_time_bits+32)) | (last_avg_time & v1_time_mask) << 32) | value | (1<<63)
        v1_out.write(v1_conv.to_bytes(8,'little'))
      FFT_index += 1
    else:
      if (after_hdr):
        FFT_index = value
        print ("(FFT index)")
        after_hdr = False
      else:
        print ("FFT, index", FFT_index, ":", real, ",", imag, "( power =", (real*real)+(imag*imag), ")")
        if (conv_2_V1):
          v1_conv = (FFT_index << (v1_time_bits+32)) | ((last_fft_time & v1_time_mask) << 32) | value
          v1_out.write(v1_conv.to_bytes(8,'little'))
        FFT_index += 1

if (conv_2_V1):
  v1_out.close()
