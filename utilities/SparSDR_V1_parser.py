#!/usr/bin/env python3

# == sample format is ==
#
# Name  |  bits
# --------------
# FFT   |     4
# Index |    12
# Time  |    16
# Real  |    16
# Imag  |    16
# ==============
# Total |    64

from sys import stdin
from struct import unpack

fft_size_log2=11

n = int(0)
fft_time_offset = int(0)
avg_time_offset = int(0)
last_fft_time = int(0)
last_avg_time = int(0)

index_mask = (2**fft_size_log2)-1
time_bits  = 32-1-fft_size_log2
time_mask  = (2**(time_bits))-1

print ("#%-9s | %9s | %9s | %9s | %9s | %9s | %9s" % ("Sample", "Type", "FFT_No", "Index", "Time(ns)", "Real", "Imag"))

while True:
  b = stdin.buffer.read(8)
  if (len(b) < 8):
    break

  (hdr, imag, real)  = unpack("Ihh", b)
  (_, avg_magnitude) = unpack("II", b)

  is_avg = (hdr >> 31) & 0x1
  index  = (hdr >> time_bits) & index_mask
  time   = (hdr & time_mask)
  fft_no = time & 0x1 # simply odd or even time for start of window

  if (n==0):
    fft_time_offset = -time
    avg_time_offset = -time
  n = n + 1

  if (is_avg):
    if (time < last_avg_time):
      avg_time_offset += (1<<time_bits)
      # print ("Average window time overflow: %d -> %d" % (last_avg_time, time))
    last_avg_time = time
    # Clock is 100mhz, and we cut (fft_size_log-1) bits to show start of window
    # Average sample times always have fft_size_log bits tail zero
    fixed_avg_time = 10 * (((time & (time_mask-1)) + avg_time_offset) << (fft_size_log2-1))
    print ("% -10d    Average                % 10d  % 10d      % 10d" % (n, index, fixed_avg_time, avg_magnitude))
    # print ("% -10d    Average                % 10d  % 10d      % 10d" % (n, index, time, avg_magnitude))

  else:
    if (time < last_fft_time):
      fft_time_offset += (1<<time_bits)
      # print ("FFT window time overflow: %d -> %d" % (last_fft_time, time))
    last_fft_time = time
    fixed_fft_time = 10 * ((time + fft_time_offset) << (fft_size_log2-1))
    print ("% -10d    FFT sample % 10d  % 10d  % 10d  % 10d  % 10d" % (n, fft_no, index, fixed_fft_time, real, imag))
    # print ("% -10d    FFT sample % 10d  %10d  % 10d  % 10d  % 10d" % (n, fft_no, index, time, real, imag))
