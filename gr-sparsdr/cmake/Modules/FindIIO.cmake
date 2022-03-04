
#
# Find the IIO includes and library
# https://github.com/analogdevicesinc/gr-iio
#
# This module exports the target IIO::IIO and the following variables:
# * IIO_FOUND
# * IIO_INCLUDE_DIR
# * IIO_VERSION
# * IIO_LIBRARIES

find_package(PkgConfig)
pkg_check_modules(PC_IIO QUIET libiio)

find_path(IIO_INCLUDE_DIR
    NAMES iio.h
    HINTS ${PC_IIO_INCLUDE_DIR}
    ${CMAKE_INSTALL_PREFIX}/include
    PATHS
    /usr/local/include
    /usr/include
)


FIND_LIBRARY(IIO_LIBRARIES
    NAMES iio
    HINTS ${PC_IIO_LIBDIR}
    ${CMAKE_INSTALL_PREFIX}/lib
    ${CMAKE_INSTALL_PREFIX}/lib64
    PATHS
    /usr/local/lib
    /usr/lib
)

set(IIO_VERSION ${PC_IIO_VERSION})
mark_as_advanced(IIO_FOUND IIO_INCLUDE_DIR IIO_LIBRARIES IIO_VERSION)

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(
    IIO
    REQUIRED_VARS IIO_INCLUDE_DIR IIO_LIBRARIES
    VERSION_VAR IIO_VERSION
)

# Create exported target
if (NOT TARGET IIO::IIO)
    add_library(IIO::IIO UNKNOWN IMPORTED)
    set_target_properties(IIO::IIO PROPERTIES
        INTERFACE_INCLUDE_DIRECTORIES "${IIO_INCLUDE_DIR}")
    set_target_properties(IIO::IIO PROPERTIES
        IMPORTED_LOCATION "${IIO_LIBRARIES}")
endif()

