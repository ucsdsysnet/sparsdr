
#
# Find the GNU Radio IIO includes and library
# https://github.com/analogdevicesinc/gr-iio
#
# This module defines
# GR_IIO_INCLUDE_DIRS
# GR_IIO_LIBRARIES
# GR_IIO_FOUND

INCLUDE(FindPkgConfig)
PKG_CHECK_MODULES(PC_GR_IIO "gnuradio-iio")

FIND_PATH(GR_IIO_INCLUDE_DIRS
    NAMES gnuradio/iio/device_source.h
    HINTS ${PC_GR_IIO_INCLUDE_DIR}
    ${CMAKE_INSTALL_PREFIX}/include
    PATHS
    /usr/local/include
    /usr/include
)

FIND_LIBRARY(GR_IIO_LIBRARIES
    NAMES gnuradio-iio
    HINTS ${PC_GR_IIO_LIBDIR}
    ${CMAKE_INSTALL_PREFIX}/lib
    ${CMAKE_INSTALL_PREFIX}/lib64
    PATHS
    /usr/local/lib
    /usr/lib
)

INCLUDE(FindPackageHandleStandardArgs)
FIND_PACKAGE_HANDLE_STANDARD_ARGS(GR_IIO DEFAULT_MSG GR_IIO_LIBRARIES GR_IIO_INCLUDE_DIRS)
MARK_AS_ADVANCED(GR_IIO_LIBRARIES GR_IIO_INCLUDE_DIRS)
