# SparSDR GNU Radio examples

These examples generally depend on `sparsdr_reconstruct` and `gr-sparsdr`
being installed according to the documentation.

## SparSDR Pluto single channel

This GNU Radio Companion file (`SparSDR Pluto single channel.grc`) receives
one Bluetooth channel from a Pluto device, reconstructs it, and displays
a waterfall graph of the reconstructed signals.

It requires an Analog Devices Pluto radio with a SparSDR v1 image installed.

### Compressing Pluto source block

The center frequency is set to 2.4 GHz. The bin specification `416..450:12800`
enables bins 416 (inclusive) to 450 (exclusive) with a threshold of 12800.
All other bins are masked (disabled). [The documentation](https://github.com/ucsdsysnet/sparsdr/blob/8cd58d7/gr-sparsdr/include/sparsdr/compressing_pluto_source.h#L147)
has more details about the bin specification format.

These bin numbers are in FFT order, not logical order. That means that bin 0
corresponds to just above the center frequency (2.4 GHz), bin 511 corresponds
to the maximum frequency (2.46144 GHz). For the other half of the bin range,
bin 512 corresponds to the minimum
frequency (2.33856 GHz) and bin 1023 corresponds to just below the center
frequency (2.4 GHz).

Bins 416 to 450 correspond to a frequency range of 2.42496 GHz to 2.427 GHz.
This is a 2.04 Mhz range centered on 2.426 GHz. That frequency corresponds
to Bluetooth Low Energy channel 38, one of the channels used for advertising.

#### Threshold notes

The threshold of 12800 is reasonable for the 34 bins that are enabled.
If you unmask more bins, you may encounter an overflow situation that makes
the radio stop sending samples. The driver will print a message if this happens.
To avoid overflow, try decreasing the gain, increasing the threshold,
or unmasking fewer bins.

### SparSDR Reconstruct block

The reconstruction block is configured with only one band. The frequency offset
of 26 MHz is relative to the center frequency of the Pluto (2.4 GHz), which adds
up to 2.426 GHz. That matches the frequency that we captured with the Pluto
block.

The number of bins is set to 64, which is the next power of two that is greater
than or equal to 34 (the number of unmasked bins in the Pluto block).

The sample rate of the reconstructed signals is (64 bins / 1024 bins) * 61.44 MHz = 3.84 MHz.
The waterfall sink uses that sample rate and a center frequency of 2.426 GHz
so that it will display the correct frequencies.
