#!/usr/bin/env python3

from sys import stdin, stdout

while True:
  b = stdin.buffer.read(4)
  if (len(b) < 4):
    break
  stdout.buffer.write(b[2:4]+b[0:2])
