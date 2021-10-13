# SparSDR patches for Pluto

To make an image that uses compressed format version 1, use the patches in the
`v1` folder. To make an image that uses compressed format version 2,
use the patches in the `v2` folder and replace `V1` in the patch file
names below with `V2`.

To compile a SparSDR Pluto image, start by cloning recursively the
[upstream repository](https://github.com/analogdevicesinc/plutosdr-fw):
`git clone --recursive https://github.com/analogdevicesinc/plutosdr-fw.git`

Follow these steps:

1. Switch to the v0.33 revision: `git checkout v0.33`
2. Update the versions of the submodules: `git submodule update`
4. Apply the patch `SparSDR_V1_Pluto_V0.33_buildroot.patch` to the `buildroot` folder
3. Apply `SparSDR_V1_Pluto_V0.33_hdl.patch` to the `hdl` folder
5. Apply `SparSDR_V1_Pluto_V0.33_linux.patch` to the `linux` folder
6. Follow the [standard instructions](https://wiki.analog.com/university/tools/pluto/building_the_image) to generate an image.

# Known issues and solutions

## Boost exported targets missing

### Symptoms

The configuration step of GNU Radio prints a few messages that look like this:

```
CMake Warning at /usr/share/cmake-3.10/Modules/FindBoost.cmake:801 (message):
  New Boost version may have incorrect or missing dependencies and imported
  targets
Call Stack (most recent call first):
  /usr/share/cmake-3.10/Modules/FindBoost.cmake:907 (_Boost_COMPONENT_DEPENDENCIES)
  /usr/share/cmake-3.10/Modules/FindBoost.cmake:1558 (_Boost_MISSING_DEPENDENCIES)
  cmake/Modules/GrBoost.cmake:75 (find_package)
  CMakeLists.txt:357 (include)
```

Later, the configuration fails after many messages that look like this:

```
CMake Error at gnuradio-runtime/lib/CMakeLists.txt:53 (add_library):
  Target "gnuradio-runtime" links to target "Boost::thread" but the target
  was not found.  Perhaps a find_package() call is missing for an IMPORTED
  target, or an ALIAS target is missing?
```

### Explanation

Buildroot compiles and installs Boost 1.72.0. GNU Radio uses CMake's
built-in `FindBoost.cmake` script to find the Boost libraries that it uses.

If the installed CMake version is too old, it does not recognize the structure
of the different Boost components and it does not create the Boost exported
targets (`Boost::thread` and similar).

### Solution

Install a newer version of CMake that recognizes version 1.72.0 of Boost.
Version 3.21.2 of CMake has been tested and works correctly. Version 3.16.2
or any newer version should also work.
