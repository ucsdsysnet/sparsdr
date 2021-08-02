# SparSDR IIO driver

This driver runs on a Pluto radio and lets a remote host change SparSDR
compression settings.

# Using a precompiled binary

If your Pluto device is running Linux 5.4.0, you can probably use the
[precompiled file](./precompiled/sparsdr_iio.ko). Skip to the "Installing"
section below.

# Compiling

## Dependencies

* Clone the Linux source from <https://github.com/analogdevicesinc/linux/>
* Check out the revision that matches the version running on your Pluto device
  (in one case, for kernel 5.4.0, the correct commit was `b05d16429dac38ecfa629c6bd9e5a403b452f57a`)
* Copy the kernel configuration file `/proc/config.gz` from the Pluto device,
  unarchive it, and put it in the Linux source directory with the name `.config`
  (this ensures that your kernel module will be compatible with the kernel on
  the Pluto device)
* Compile Linux following [the instructions](https://wiki.analog.com/resources/tools-software/linux-build/generic/zynq)

## Actually compiling

* Edit the Makefile, changing `KDIR` to the linux folder where you compiled
  the kernel
* Set up cross-compilation with these commands:
    * `export ARCH=arm`
    * `export CROSS_COMPILE=[the path to your ARM compiler binaries, including the arm-linux-gnueabi- prefix]`
      (example: `/some-folder/pluto/gcc-linaro-5.5.0-2017.10-x86_64_arm-linux-gnueabi/bin/arm-linux-gnueabi-`)
* Compile the kernel module by running `make`

# Installing

* Copy the `sparsdr_iio.ko` file onto the Pluto device using `scp`
* SSH to the Pluto device, navigate to the folder containing the
  `sparsdr_iio.ko` file, and run `insmod sparsdr_iio.ko`

At this point, if you run `iio_info` on the Pluto, a `sparsdr` device with
several attributes should appear at the end of the output.

The `sparsdr` IIO device can be controlled locally, but not from other
computers. Restart the IIO server so that it detects `sparsdr`:

* `/etc/init.d/S23udc reload`

This command will also restart the network interface, so you may need to
reconnect to the Pluto device from your computer. You may need to assign a
static IP address of 192.168.2.2 to your computer on the Pluto virtual network
interface.

After that, `iio_info -u ip:192.168.2.1` on the computer should be able to
connect to the Pluto device and show the SparSDR attributes.

# Using

The interface that this driver provides is mainly an implementation detail.
It is not currently documented very well, and it may change.

## Examples

* Read the FFT size from a host computer: `iio_attr -u ip:192.168.2.1 -d sparsdr fft_size`
* Set the FFT size from a host computer: `iio_attr -u ip:192.168.2.1 -d sparsdr fft_size 512`
