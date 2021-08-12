# SparSDR

This code accompanies the paper [*SparSDR: Sparsity-proportional Backhaul and Compute for SDRs*](https://cseweb.ucsd.edu/~schulman/docs/mobisys19-sparsdr.pdf).

## What's included

* An FPGA image for the USRP N210 that captures 100 MHz of bandwidth and
sends compressed signals
* The `sparsdr_reconstruct` program, which reconstructs signals from compressed
data
* The `gr-sparsdr` module for GNU Radio, which makes SparSDR easy to use

![GNU Radio Companion screenshot](doc/images/grc_screenshot.png)

## Getting started

See [the getting started guide](doc/getting_started.md).

## Debugging

When SparSDR does not work correctly, [the debugging guide](doc/debugging.md) may help.

## Compatibility

We have tested SparSDR with this configuration:

* GNU Radio 3.7.13.4, 3.7.13.5, and 3.7.14.1
* UHD 3.10.3.0, 3.13.1.0, and 3.15.0.0
* Hardware: USRP N210 revision 4 with an SBX-120 or a CBX-120 daughter board.

SparSDR should work with any GNU Radio version in the 3.7 series. GNU Radio 3.8 has some changes that prevent the SparSDR module from compiling.

This repository includes one FPGA image for the USRP N210 revision 4, and a separate FPGA image for N210 revisions 2 and 3.

Daughter boards other than the SBX-120 should also work, with the following considerations:

* If the daughter board receive bandwidth is less than 100 MHz, this will limit what you can receive with SparSDR
* If the daughter board receive bandwidth is greater than 100 MHz, you may see aliasing because the signal will still be sampled at 100 megasamples per second.

## Pluto compatibility

SparSDR also works with the Analog Devices ADALM-PLUTO radio, with a different
version of the reconstruct application and a new Compressing Pluto Source block
for GNU Radio. See [the documentation in the pluto branch](https://github.com/ucsdsysnet/sparsdr/blob/pluto/doc/pluto.md)
for more details.

## Licenses

* `gr-sparsdr`: GNU GPL v3 or later
* `fpga_images/Pluto`: GNU GPL 2
* `fpga_src/Pluto`: GNU GPL v2 or Apache 2.0
* `fpga_images/N210` and `fpga_src/N210`: GNU GPL v3 or later
* Everything else: Apache 2.0
