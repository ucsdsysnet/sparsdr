#!/usr/bin/env python3

from sys import stdin
from struct import unpack

while True:
  b = stdin.buffer.read(4)
  if (len(b) < 4):
    break

  (imag, real) = unpack("hh", b)
  print (real, ",", imag)
