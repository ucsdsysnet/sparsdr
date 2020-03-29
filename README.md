# SparSDR

This code accompanies the paper [SparSDR: Sparsity-proportional Backhaul and Compute for SDRs](http://cseweb.ucsd.edu/~schulman/docs/mobisys19-sparsdr.pdf).

## What's included

* An FPGA image for the USRP N210 that captures 100 MHz of bandwidth and
sends compressed signals
* The `sparsdr_reconstruct` program, which reconstructs signals from compressed
data
* The `gr-sparsdr` module for GNU Radio, which makes SparSDR easy to use

![GNU Radio Companion screenshot](doc/images/grc_screenshot.png)

## Getting started

See [the getting started guide](doc/getting_started.md).

## Compatibility

SparSDR is currently tested with GNU Radio 3.7.13.4. It should work with any
version in the 3.7 series. Version 3.9 has some changes that break compatibility
with SparSDR

## Licenses

* `gr-sparsdr`: GNU GPL v3 or later
* `fpga_images/Pluto`: GNU GPL 2
* `fpga_src/Pluto`: GNU GPL v2 or Apache 2.0
* `fpga_images/N210` and `fpga_src/N210`: GNU GPL v3 or later
* Everything else: Apache 2.0
