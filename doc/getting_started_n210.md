# SparSDR USRP N210 getting started

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

FPGA images are in the `fpga_images/N210` folder of this repository.
There are two subfolders for the different compressed sample formats. The v1 format is better for narrow signals like AM
audio, and the v2 format is better for wide signals like Bluetooth.
Within each subfolder, one image file is for revisions 2 and 3 of the USRP N210 and the other file is for revision 4.

Run `uhd_image_loader --args 'type=usrp2' --fpga-path SparSDR_N210_[revision].bin`

When that completes, power cycle the USRP to start using the new image.

## Using command-line tools

SparSDR provides simple command-line tools to receive signals and reconstruct
them. See [the command-line tool documentation](command_line.md) for more
details.

## Using the blocks in GNU Radio Companion

SparSDR provides some GNU Radio blocks for simple receiving and reconstruction.
Both blocks can be configured and used through GNU Radio Companion.

### Compressing USRP source

The compressing USRP source configures the USRP to use SparSDR compression, and reads compressed samples.
The output of this block is a stream of compressed samples that can be saved to a file or reconstructed in real time.

Options:
* Device Address: A value like `addr=192.168.10.1` specifies the IP address of the USRP. If this is empty,
  the driver will look for a USRP and connect to one automatically.
* Center frequency, gain, and antenna: These are the same as the corresponding options on the normal UHD USRP source
  block.
* Threshold: This determines which bins will be sent
  from the USRP to the host for processing. If the threshold is too low, many
  bins will be sent and overflow will happen. If the threshold is too low,
  not enough bins will be sent and the signals will probably not be decodable.

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

### USRP Receive and Reconstruct

This block combines the Compressing USRP Source and a SparSDR Reconstruct blocks into one block that is
simpler to use. It has one or more outputs with 32-bit complex float time-domain samples.

The options for this block are the same as described in the previous sections.
