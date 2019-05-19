# SparSDR

This code accompanies the paper *SparSDR: Sparsity-proportional Backhaul and Compute for SDRs*.

## What's included

* An FPGA image for the USRP N210 that captures 100 MHz of bandwidth and
sends compressed signals
* The `sparsdr_reconstruct` program, which reconstructs signals from compressed
data
* The `gr-sparsdr` module for GNU Radio, which makes SparSDR easy to use

![GNU Radio Companion screenshot](doc/images/grc_screenshot.png)

## Getting started

See [the getting started guide](doc/getting_started.md).

## Licenses

* `gr-sparsdr`: GNU GPL v3 or later
* Everything else: Apache license 2.0
