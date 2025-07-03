find_package(PkgConfig)

PKG_CHECK_MODULES(PC_GR_SPARSDR gnuradio-sparsdr)

FIND_PATH(
    GR_SPARSDR_INCLUDE_DIRS
    NAMES gnuradio/sparsdr/api.h
    HINTS $ENV{SPARSDR_DIR}/include
        ${PC_SPARSDR_INCLUDEDIR}
    PATHS ${CMAKE_INSTALL_PREFIX}/include
          /usr/local/include
          /usr/include
)

FIND_LIBRARY(
    GR_SPARSDR_LIBRARIES
    NAMES gnuradio-sparsdr
    HINTS $ENV{SPARSDR_DIR}/lib
        ${PC_SPARSDR_LIBDIR}
    PATHS ${CMAKE_INSTALL_PREFIX}/lib
          ${CMAKE_INSTALL_PREFIX}/lib64
          /usr/local/lib
          /usr/local/lib64
          /usr/lib
          /usr/lib64
          )

include("${CMAKE_CURRENT_LIST_DIR}/gnuradio-sparsdrTarget.cmake")

INCLUDE(FindPackageHandleStandardArgs)
FIND_PACKAGE_HANDLE_STANDARD_ARGS(GR_SPARSDR DEFAULT_MSG GR_SPARSDR_LIBRARIES GR_SPARSDR_INCLUDE_DIRS)
MARK_AS_ADVANCED(GR_SPARSDR_LIBRARIES GR_SPARSDR_INCLUDE_DIRS)
