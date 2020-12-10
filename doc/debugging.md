# Debugging SparSDR

Problems with SparSDR are most likely to happen when the radio is not configured correctly. This document describes some debugging strategies that may be useful.

## Receive compressed samples to a file

By receiving some compressed samples and looking at them, you can see if they look reasonable. There are two ways to write the samples to a file. They should both work the same and produce similar-looking files.

### Using `sparsdr_receive`

Run `sparsdr_receive --antenna=RX2 --output-path=sample_capture.iqz --threshold=5000 --gain=30 --frequency=2450000000 --mask-bins 0..1024` (stop it after a few seconds).

### Using `uhd_rx_compressed_cfile`

Use the Python script, located at `examples/uhd_rx_compressed_cfile/uhd_rx_compressed_cfile` in this repository: `uhd_compressed_rx_cfile -f 2.45e9 -g 30 --threshold 5000 -A RX2 -s Raw_SparSDR_capture`

### Inspecting compressed samples

Use the Python script located at `utilities/fpga_compress_with_avg_print.py` to display samples in a human-readable format: `fpga_compress_with_avg_print.py < SparSDR_capture > SparSDR_capture_readable`

To get a sense of what the samples should look like, try running `fpga_compress_with_avg_print.py` on one of the example files in this repository: `examples/sparsdr_receive/sample_capture.iqz` or `examples/uhd_rx_compressed_cfile/Raw_SparSDR_capture`. A normal file contains groups of average samples and FFT (data) samples. The FFT samples within a group are sorted by the bit-reversal of the FFT index.

## Reconstruct compressed samples

You can use `sparsdr_reconstruct` to read compressed samples from a file, reconstruct the signals, and write them to another file. Here is an example command: `sparsdr_reconstruct --bins 63 --center-frequency=30e6 --source Raw_SparSDR_capture --destination reconstructed_capture`. The [command line documentation](command_line.md) gives full details about the options.

