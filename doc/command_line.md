# SparSDR command-line tools

## Receive compressed signals: `sparsdr_receive`

`sparsdr_receive` configures a USRP for compression, receives compressed signals,
and writes them to a file.

The most frequently used command-line options are:

* `--antenna`: The antenna to use (with a USRP N210, valid values are `TX/RX`
    and `RX2`)
* `--output-path`: The path to the output file to write
* `--threshold`: The signal level threshold to use for compression
    (for more details on threshold and gain setting, see the
    [threshold and gain guide](threshold_gain.md))
* `--gain`: The receiver gain, in decibels
* `--frequency`: The center frequency to capture, in hertz

For a complete and up-to-date list of options, run `sparsdr_receive --help`.

### Overflow

`sparsdr_receive` detects overflow and prints the message "Compression internal overflow, restarting."

## Reconstruct signals: `sparsdr_reconstruct`

`sparsdr_reconstruct` decompresses SparSDR compressed files. It can be used
offline with regular files for input and output, or in real-time using standard
input/standard output or named pipes.

The most frequently used command-line options are:

* `--bins`: The number of bins to decompress, out of a maximum of 2048
* `--center-frequency`: The desired center frequency of the decompressed signal,
    relative to the center frequency of the compressed data
* `--source`: The path to the compressed file to read (produced by `sparsdr_receive`)
* `--destination`: The path to the uncompressed file to write. This output
    file will contain samples in the standard GNU Radio complex format, with
    32-bit floating-point real followed by 32-bit floating point imaginary
    for each sample.

For a complete and up-to-date list of options, run `sparsdr_reconstruct --help`.
Advanced options are available to decompress more than one frequency band
at the same time.
