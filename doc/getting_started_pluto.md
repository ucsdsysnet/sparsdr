# SparSDR Pluto getting started

## Dependencies

* A C/C++ compiler
* [CMake](https://cmake.org/) (tested with version 3.7.2)
* [GNU Radio](https://www.gnuradio.org/) (tested with version 3.7.10.1)
* The [Rust compiler](https://www.rust-lang.org/learn/get-started) (latest stable version)
* [FFTW](http://www.fftw.org/) (tested with version 3.3.5)
* [SWIG](http://www.swig.org/) for generating Python bindings (tested with version 3.0.10)
* [libiio](https://github.com/analogdevicesinc/libiio) for PlutoSDR Compatibility
* [libad9361-iio](https://github.com/analogdevicesinc/libad9361-iio) for PlutoSDR Compatibility
* [gr-iio](https://github.com/analogdevicesinc/gr-iio) for PlutoSDR GNURadio Bindings

### Installing dependencies on Ubuntu

```
sudo apt install build-essential cmake gnuradio libfftw3-3 swig libad9361-0 libiio-dev gr-iio
curl https://sh.rustup.rs -sSf | sh
source $HOME/.cargo/env
```

The second command runs the Rust compiler installer. Use the default
installation options.

## Installing `sparsdr_reconstruct`

In the `reconstruct` folder, run `cargo install --path .`

## Installing `gr-sparsdr`

The `gr-sparsdr` module uses the standard CMake build process. From the
`gr-sparsdr` folder, run:

```
mkdir build
cd build
cmake ..
make
sudo make install
```

## Loading the FPGA image

FPGA images are in the `fpga_images/Pluto` folder of this repository.
There are two subfolders for the different compressed sample formats. The v1 format is better for narrow signals like AM
audio, and the v2 format is better for wide signals like Bluetooth.

Follow [the instructions from Analog Devices](https://wiki.analog.com/university/tools/pluto/common/firmware)
to install the image.

## Using the blocks in GNU Radio Companion

SparSDR provides some GNU Radio blocks for simple receiving and reconstruction.
Both blocks can be configured and used through GNU Radio Companion.

### Compressing Pluto source

The compressing Pluto source configures the Pluto and reads compressed samples.
The output of this block is a stream of compressed samples that can be saved to a file or reconstructed in real time.

Options:
* IIO context URI: The default `ip:192.168.2.1` works in most cases. Other values, as specified in the libiio
  documentation, may sometimes be needed.
* Bin specification: This determines which of the 1024 FFT bins will be unmasked (allowing signals to be received in the
  corresponding frequency ranges) and what threshold should be set for each bin.

  A bin specification contains zero or more threshold groups, separated by commas.

  A threshold group contains one bin range, a colon `:`, and one threshold value.

  A bin range can be a single bin number, or two bin numbers separated by
  two periods `..`. If two numbers are provided, they represent a range
  of bins. The start of the range is included, and the end of the range
  is excluded.

  A threshold value is a non-negative integer.

  Any bins not specified will be masked (preventing them from sending any samples).

  Examples:
  * Mask all bins: (empty string)
  * Enable bin 42 with a threshold of 4000: `42:4000`
  * Enable bins 100 (inclusive) to 200 (exclusive) with a threshold of 800: `100..200:800`
  * Enable bins 1000 and 1020, both with a threshold of 8192:`1000:8192,1020:8192`

* Average sample interval: This determines how frequently the FPGA sends average samples. After this many FFT
  operations, a set of averages will be sent. This value must be a power of two. The default
  value of `2 ** 20` is usually correct.
* Center frequency and gain: These are the same as the corresponding options on the normal Pluto source
  block.
* Buffer size (samples): This is the size of buffers that the IIO library uses. Smaller buffers will reduce latency
  but increase the probability that compressed samples will get dropped.
* Shift amount: This determines the number of bits of shift that the FPGA applies to bin values.
* FFT size: This determines the number of bins that the FPGA uses to compress signals. It must be a power of
  two and may not be greater than 1024.

### SparSDR Reconstruct

This block runs the sparsdr_reconstruct program to convert compressed samples into time-domain samples.
It has one input that takes compressed samples, and one or more outputs with 32-bit complex float time-domain samples.

Options:
* Executable: The default value of `sparsdr_reconstruct` will find the reconstruction program in the system PATH.
  This can also be a path to a different version of the reconstruction program.
* Bands: This is the number of different frequency ranges to reconstruct. Each band has its own output stream
  and can correspond to a different frequency range.

  Each band has a frequency parameter, which is the center frequency to reconstruct
  relative to the center frequency used to capture signals. Each band also has
  a bins parameter, which specifies the number of FFT bins to reconstruct out of
  the 2048 bins used when receiving the original 100 MHz. As an example,
  if the USRP were capturing at a center frequency of 2.5 GHz,
  a band with an offset of -50 Mhz and 1024 bins would reconstruct half of the
  original signal in the frequency range 2.0 to 2.5 GHz.

* Compressed format: Select the appropriate option for the radio used to receive the samples and the sample format
  version (determined by the FPGA image)
* Zero samples in gaps: With this disabled, the reconstruction program will not produce any output samples for times
  when there were no active signals (above the threshold) to receive. This means that the transmissions are compressed
  together in time with no gaps.
  With this option enabled, the number of samples in the output file will match the time spent receiving signals,
  and the times of signals will be accurate. The output file will likely be much larger.
* Bands: This tab provides the options for each band.
  * Band *n* frequency: The center frequency to reconstruct, relative to the center frequency used to receive the
    signals
  * Band *n* bins: The number of bins to reconstruct. This determines the bandwidth and sample rate of the file:
    bandwidth = (number of bins / 2048) * 100 megahzertz

#### Fixing the PATH

The Rust compiler installer adds `~/.cargo/bin` to `$PATH`, and `sparsdr_reconstruct` gets installed
there. When the SparSDR Reconstruct block runs, it will not be able to find `sparsdr_reconstruct`.
There are two ways to fix this:

* Option 1: Log out and then log back in, so that GNU Radio Companion will use the updated `PATH`
* Option 2: In the SparSDR reconstruct block properties, change the Executable to
`/home/<username>/.cargo/bin/sparsdr_reconstruct`

### Pluto Receive and Reconstruct

This block combines the Compressing Pluto Source and a SparSDR Reconstruct blocks into one block that is
simpler to use. It has one or more outputs with 32-bit complex float time-domain samples.

The options for this block are the same as described in the previous sections.
