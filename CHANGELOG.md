# SparSDR Changelog

## 2021-10-13 Pluto version

### Pluto images

 * Added support for Pluto hardware revision C
 * Fixed an issue that caused FFT samples for indexes 512 through 520 to have
   timestamp values one less than expected (format version 1)
 * Fixed an issue that corrupted the compressed samples (format version 2)
 * Moved the SparSDR IIO driver into the kernel (this eliminates the need to
   install a separate kernel module each time the Pluto device starts up)

### gr-sparsdr

 * Added more options to the compressing Pluto source block
 * Removed GRC block XML for some blocks that are not complete

### Reconstruct

 * Fixed an issue that could cause window timestamps to be incorrect
 * Fixed some test cases to match the Pluto configuration
