
#
# Find the FFTW (Fastest Fourier Transform in the West) includes and library
# https://fftw.org/
#
# This module exports the targes FFTW::FFTW3 and FFTW:FFTW3F and the following variables:
# * FFTW_FOUND
# * FFTW_INCLUDE_DIR
# * FFTW_VERSION
# * FFTW3_LIBRARIES
# * FFTW3F_LIBRARIES

find_package(PkgConfig)
pkg_check_modules(PC_FFTW3 QUIET fftw3)
pkg_check_modules(PC_FFTW3F QUIET fftw3f)

find_path(FFTW_INCLUDE_DIR
    NAMES fftw3.h
    HINTS ${PC_FFTW3_INCLUDE_DIR}
    ${PC_FFTW3F_INCLUDE_DIR}
    ${CMAKE_INSTALL_PREFIX}/include
    PATHS
    /usr/local/include
    /usr/include
)


FIND_LIBRARY(FFTW3_LIBRARIES
    NAMES fftw3
    HINTS ${PC_FFTW3_LIBDIR}
    ${CMAKE_INSTALL_PREFIX}/lib
    ${CMAKE_INSTALL_PREFIX}/lib64
    PATHS
    /usr/local/lib
    /usr/lib
)
FIND_LIBRARY(FFTW3F_LIBRARIES
    NAMES fftw3f
    HINTS ${PC_FFTW3f_LIBDIR}
    ${CMAKE_INSTALL_PREFIX}/lib
    ${CMAKE_INSTALL_PREFIX}/lib64
    PATHS
    /usr/local/lib
    /usr/lib
)

set(FFTW_VERSION ${PC_FFTW3_VERSION})
mark_as_advanced(FFTW_FOUND FFTW_INCLUDE_DIR FFTW3_LIBRARIES FFTW3F_LIBRARIES FFTW_VERSION)

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(
    FFTW
    REQUIRED_VARS FFTW_INCLUDE_DIR FFTW3_LIBRARIES FFTW3F_LIBRARIES
    VERSION_VAR FFTW_VERSION
)

# Create exported targets
if (NOT TARGET FFTW::FFTW3)
    add_library(FFTW::FFTW3 UNKNOWN IMPORTED)
    set_target_properties(FFTW::FFTW3 PROPERTIES
        INTERFACE_INCLUDE_DIRECTORIES "${FFTW_INCLUDE_DIR}")
    set_target_properties(FFTW::FFTW3 PROPERTIES
        IMPORTED_LOCATION "${FFTW3_LIBRARIES}")
endif()

if (NOT TARGET FFTW::FFTW3F)
    add_library(FFTW::FFTW3F UNKNOWN IMPORTED)
    set_target_properties(FFTW::FFTW3F PROPERTIES
        INTERFACE_INCLUDE_DIRECTORIES "${FFTW_INCLUDE_DIR}")
    set_target_properties(FFTW::FFTW3F PROPERTIES
        IMPORTED_LOCATION "${FFTW3F_LIBRARIES}")
endif()


