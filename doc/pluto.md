# SparSDR with a Pluto software-defined radio

SparSDR can run on an [Analog Devices ADALM-PLUTO](https://www.analog.com/en/design-center/evaluation-hardware-and-software/evaluation-boards-kits/ADALM-PLUTO.html)
software-defined radio to receive signals. The Pluto device is much less
expensive than a USRP N210 ($150 instead of about $2500) and provides similar
functionality.

The main differences of the Pluto implementation of SparSDR are:

* Bandwidth 20 MHz instead of 100 MHz
* Different tuning range (325 MHz to 3.8 GHz)
* Sample rate 61.44 MHz instead of 100 MHz
* 1024 FFT bins instead of 2048
* The compressed sample format is slightly different

Because the compressed sample format is different, you'll need a different
version of the `sparsdr_reconstruct` application to reconstruct the received
signals. That version, and all the other Pluto tools, are in the `pluto` branch
of this repository.

## Setup instructions

### Installing the SparSDR firmware and FPGA image

Use the firmware and FPGA image in this repository: TODO (add the file)

Install the `pluto.frm` file on your Pluto device. The easiest way to do this
is:

* Copy `pluto.frm` to the `PlutoSDR` external storage volume
* Eject the `PlutoSDR` external storage volume
* Wait for the Pluto to install the firmware and restart

For more details, see the [full firmware update instructions](https://wiki.analog.com/university/tools/pluto/users/firmware).

After installing the firmware, the login prompt should show the firmware version
as `v0.30-dirty`, for example:

```
Welcome to:
______ _       _        _________________
| ___ \ |     | |      /  ___|  _  \ ___ \
| |_/ / |_   _| |_ ___ \ `--.| | | | |_/ /
|  __/| | | | | __/ _ \ `--. \ | | |    /
| |   | | |_| | || (_) /\__/ / |/ /| |\ \
\_|   |_|\__,_|\__\___/\____/|___/ \_| \_|

v0.30-dirty
http://wiki.analog.com/university/tools/pluto
```

### Installing the SparSDR IIO driver

Follow the [SparSDR IIO instructions](../pluto_sparsdr_iio/README.md) to install the SparSDR IIO driver.

After completing those steps, if you run `iio_info -u ip:192.168.2.1` on the
host computer, you should see a long list of devices and attributes ending with
something like this:

```
iio:device5: sparsdr
    0 channels found:
    12 device-specific attributes found:
            attr  0: average_interval value: 16
            attr  1: average_weight value: 224
            attr  2: bin_mask value: 0
            attr  3: enable_compression value: 1
            attr  4: fft_scaling value: 1707
            attr  5: fft_size value: 10
            attr  6: run_fft value: 1
            attr  7: send_average_samples value: 1
            attr  8: send_fft_samples value: 1
            attr  9: threshold_bin_number value: 0
            attr 10: threshold_value value: 0
            attr 11: window_value value: 0
    No trigger on this device
```

Important note: After restarting the Pluto device, you will need to perform the
"Installing the SparSDR IIO driver" steps again.

### Receiving signals
