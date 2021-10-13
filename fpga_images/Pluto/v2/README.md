This image can be installed on an [Analog Devices ADALM-Pluto](https://www.analog.com/en/design-center/evaluation-hardware-and-software/evaluation-boards-kits/ADALM-PLUTO.html)
radio (hardware revision B or C) to enable SparSDR.

Follow [the instructions from Analog Devices](https://wiki.analog.com/university/tools/pluto/common/firmware)
to install the image.

This image (v2) sends compressed samples to the host using version 2 of the
sample format. The reconstruction support for this format version is not yet
finished (as of 2021-10-13). If in doubt, use the other (v1) image.

For patches and compiling instructions, see [the source folder](../../../fpga_src/Pluto).
