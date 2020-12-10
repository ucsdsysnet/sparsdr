# SparSDR getting started

## Dependencies

* A C/C++ compiler
* [CMake](https://cmake.org/) (tested with version 3.7.2)
* [GNU Radio](https://www.gnuradio.org/) (tested with version 3.7.10.1)
* [UHD](https://github.com/EttusResearch/uhd/) library and headers (tested with version 3.9.5)
* The [Rust compiler](https://www.rust-lang.org/learn/get-started) (latest stable version)
* [FFTW](http://www.fftw.org/) (tested with version 3.3.5)
* [SWIG](http://www.swig.org/) for generating Python bindings (tested with version 3.0.10)

### Installing dependencies on Ubuntu

```
sudo apt install build-essential cmake gnuradio libuhd-dev libfftw3-3 swig
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

FPGA images are in the `fpga_images/N210` folder of this repository. One image file is for revisions 2 and 3 of the USRP N210, and the other file is for revision 4.

Run `uhd_image_loader --args 'type=usrp2' --fpga-path SparSDR_N210_[revision].bin`

When that completes, power cycle the USRP to start using the new image.

## Using command-line tools

SparSDR provides simple command-line tools to receive signals and reconstruct
them. See [the command-line tool documentation](command_line.md) for more
details.

## Using the blocks in GNU Radio Companion

SparSDR provides two GNU Radio blocks for simple receiving and reconstruction.
Both blocks can be configured and used through GNU Radio Companion.

### Compressing USRP source

The compressing USRP source configures the USRP to use SparSDR compression.
The center frequency and gain parameters are the same as with the standard
USRP source block. The device address parameter can usually be left empty,
but the IP address of the USRP can be specified with the syntax
`addr=192.168.10.1`. The threshold parameter determines which bins will be sent
from the USRP to the host for processing. If the threshold is too low, many
bins will be sent and overflow will happen. If the threshold is too low,
not enought bins will be sent for the signals to be decoded.


#### Fixing the PATH

The Rust compiler installer adds `~/.cargo/bin` to `$PATH`, and `sparsdr_reconstruct` gets installed
there. When the SparSDR Reconstruct block runs, it will not be able to find `sparsdr_reconstruct`.
There are two ways to fix this:

* Option 1: Log out and then log back in, so that GNU Radio Companion will use the updated `PATH`
* Option 2: In the SparSDR reconstruct block properties, change the Executable to
`/home/<username>/.cargo/bin/sparsdr_reconstruct`

### SparSDR Reconstruct

The reconstruct block reconstructs compressed signals in one or more bands.
Each band has a frequency parameter, which is the center frequency to reconstruct
relative to the center frequency used to capture signals. Each band also has
a bins parameter, which specifies the number of FFT bins to reconstruct out of
the 2048 bins used when receiving the original 100 MHz. As an example,
if the USRP were capturing at a center frequency of 2.5 GHz,
a band with an offset of -50 Mhz and 1024 bins would reconstruct half of the
original signal in the frequency range 2.0 to 2.5 GHz.

### Sample rates

The actual sample rate of the reconstructed signals depends on the number of
bins used for reconstruction, which may be greater than the numbers of bins
specified in the band setup.

To calculate the actual sample rate of an output, let `rounded_bins`
be the number of bins rounded up to the nearest power of two. The sample rate
will be `rounded_bins / 2048 * 100 MS/s`.
