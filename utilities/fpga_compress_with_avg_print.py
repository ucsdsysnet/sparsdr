#!/usr/bin/env python

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

n = 0
time_offset = 0L
last_time = 0L
time = 0L
avg = 0L
fixed_time = 0L
fft_size_log2=11

print "#%-9s | %9s | %9s | %9s | %9s | %9s | %9s" % ("Sample", "Type", "FFT_No", "Index", "Time(ns)", "Real", "Imag")

while True:
  b = stdin.read(8)
  if (len(b) < 8):
    break

  (fft_index, time, real, imag) = unpack("HHhh", b)
  (_, _, mag_msb, mag_lsb)      = unpack("HHHH", b)
  is_avg = (fft_index >> 15) & 0x1
  index = (fft_index>>4) & 0x7FF
  time = ((fft_index & 0xF)<<16)+time;
  avg_magnitude = (mag_msb<<16)+mag_lsb
  fft_no = time & 0x1 # simply odd or even time for start of window

  if (n==0):
    time_offset = -time
  n = n + 1

  if (time < last_time) and (not is_avg):
    # print "overflow: %d -> %d" % (last_time, time)
    time_offset += (1<<20)

  # We add index for the lower bits of time
  fixed_time = 10L * (((time + time_offset)<<(fft_size_log2-1)) + index)
  # Average samples do not take part in overflow detection
  if (not is_avg):
    last_time = time

  if (is_avg):
    print "% -10d    Average                % 10d  % 10d      % 10d" % (n, index, fixed_time, avg_magnitude)
    # print "% -10d    Average                % 10d  % 10d      % 10d" % (n, index, time, avg_magnitude)
  else:
    print "% -10d    FFT sample % 10d  % 10d  % 10d  % 10d  % 10d" % (n, fft_no, index, fixed_time, real, imag)
    # print "% -10d    FFT sample % 10d  %10d  % 10d  % 10d  % 10d" % (n, fft_no, index, time, real, imag)
    # avg += ((real*real) + (imag*imag))

# print "Avg magnitude over %d samples is %d" % (n, avg*1.0/n)
