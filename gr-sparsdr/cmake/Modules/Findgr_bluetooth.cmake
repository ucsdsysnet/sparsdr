
#
# Find the gr_bluetooth includes and library
# https://github.com/greatscottgadgets/gr-bluetooth
#
# This module defines
# GR_BLUETOOTH_INCLUDE_DIRS
# GR_BLUETOOTH_LIBRARIES
# GR_BLUETOOTH_FOUND

INCLUDE(FindPkgConfig)
PKG_CHECK_MODULES(PC_GR_BLUETOOTH "gr_bluetooth")

FIND_PATH(GR_BLUETOOTH_INCLUDE_DIRS
    NAMES gr_bluetooth/multi_sniffer.h
    HINTS ${PC_GR_BLUETOOTH_INCLUDE_DIR}
    ${CMAKE_INSTALL_PREFIX}/include
    PATHS
    /usr/local/include
    /usr/include
)

FIND_LIBRARY(GR_BLUETOOTH_LIBRARIES
    NAMES gnuradio-bluetooth
    HINTS ${PC_GR_BLUETOOTH_LIBDIR}
    ${CMAKE_INSTALL_PREFIX}/lib
    ${CMAKE_INSTALL_PREFIX}/lib64
    PATHS
    ${CPPUNIT_INCLUDE_DIRS}/../lib
    /usr/local/lib
    /usr/lib
)

INCLUDE(FindPackageHandleStandardArgs)
FIND_PACKAGE_HANDLE_STANDARD_ARGS(GR_BLUETOOTH DEFAULT_MSG GR_BLUETOOTH_LIBRARIES GR_BLUETOOTH_INCLUDE_DIRS)
MARK_AS_ADVANCED(GR_BLUETOOTH_LIBRARIES GR_BLUETOOTH_INCLUDE_DIRS)
