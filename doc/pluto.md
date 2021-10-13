# SparSDR with a Pluto software-defined radio

SparSDR can run on an [Analog Devices ADALM-PLUTO](https://www.analog.com/en/design-center/evaluation-hardware-and-software/evaluation-boards-kits/ADALM-PLUTO.html)
software-defined radio (hardware revision B or C) to receive signals. The Pluto
device is much less expensive than a USRP N210 ($150 instead of about $2500) and
provides similar functionality.

The main differences of the Pluto implementation of SparSDR are:

* Bandwidth 20 MHz instead of 100 MHz
* Different tuning range (325 MHz to 3.8 GHz)
* Sample rate 61.44 MHz instead of 100 MHz
* 1024 FFT bins instead of 2048
* The compressed sample format (version 1) is slightly different

Because the compressed sample format is different, you'll need a different
version of the `sparsdr_reconstruct` application to reconstruct the received
signals. That version, and all the other Pluto tools, are in the `pluto` branch
of this repository.

## Setup instructions

### Installing the SparSDR firmware and FPGA image

Use [the firmware and FPGA image in this repository](../fpga_images/Pluto/v1/)

Install the `pluto.frm` file on your Pluto device. The easiest way to do this
is:

* Copy `pluto.frm` to the `PlutoSDR` external storage volume
* Eject the `PlutoSDR` external storage volume
* Wait for the Pluto to install the firmware and restart

For more details, see the [full firmware update instructions](https://wiki.analog.com/university/tools/pluto/users/firmware).

After installing the firmware, the login prompt should show the firmware version
as `v0.33-3-gd382-dirty`, for example:

```
Welcome to:
______ _       _        _________________
| ___ \ |     | |      /  ___|  _  \ ___ \
| |_/ / |_   _| |_ ___ \ `--.| | | | |_/ /
|  __/| | | | | __/ _ \ `--. \ | | |    /
| |   | | |_| | || (_) /\__/ / |/ /| |\ \
\_|   |_|\__,_|\__\___/\____/|___/ \_| \_|

v0.33-3-gd382-dirty
http://wiki.analog.com/university/tools/pluto
```

### Receiving signals

Use the GNU Radio Compressing Pluto Source Block to read samples from the Pluto
device.

The block has these options:

* IIO context URI: The URI to use when connecting to the Pluto device. This is
  normally `ip:192.168.2.1`, but other IP addresses or a USB URI can be used if
  necessary.
* Bin specification: This string specifies the bins that you want to receive,
  and the threshold for each range of bins. Any bin that is not specified here
  will be masked, so you will not get any samples from that bin.
  Some example bin specifications:
   * Mask all bins: (empty string)
   * Enable bin 42 with a threshold of 4000: `42:4000`
   * Enable bins 0, 1, 2, 3, 4, 5, 6, and 7 with a threshold of 1: `0..8:1`
   * Enable bins 100 (inclusive) to 200 (exclusive) with a threshold
     of 800: `100..200:800`
   * Enable bins 1000 and 1020, both with a threshold of 8192:
     `1000:8192,1020:8192`
 * Average sample interval: The interval between sets of average samples sent
   by the Pluto device. This is normally `2 ** 20`, but can be higher or lower.
 * Center frequency: The frequency to tune the radio to, in hertz
 * Gain: The radio receiver gain, in decibels
 * Buffer size: The size of the buffers used to hold compressed samples.
   Values lower than 1024 * 1024 increase the risk of silently dropping samples.
 * Shift amount: The amount that values are shifted in the FFT.
   Lower values increase the risk of numerical overflow, but allow more
   precision with weak signals.
 * FFT size: The size of the FFT, in bins. As of 2021-10-13 the reconstruction
   application only works when this is set to 1024.
