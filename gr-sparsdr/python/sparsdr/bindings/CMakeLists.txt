# Copyright 2020 Free Software Foundation, Inc.
#
# This file is part of GNU Radio
#
# SPDX-License-Identifier: GPL-3.0-or-later
#

########################################################################
# Check if there is C++ code at all
########################################################################
if(NOT sparsdr_sources)
    message(STATUS "No C++ sources... skipping python bindings")
    return()
endif(NOT sparsdr_sources)

########################################################################
# Check for pygccxml
########################################################################
gr_python_check_module_raw("pygccxml" "import pygccxml" PYGCCXML_FOUND)

include(GrPybind)

########################################################################
# Python Bindings
########################################################################

list(APPEND sparsdr_python_files python_bindings.cc)

gr_pybind_make_oot(sparsdr ../../.. gr::sparsdr "${sparsdr_python_files}")

# copy bindings extension for use in QA test module
add_custom_command(
    TARGET sparsdr_python
    POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:sparsdr_python>
            ${PROJECT_BINARY_DIR}/test_modules/gnuradio/sparsdr/)

install(
    TARGETS sparsdr_python
    DESTINATION ${GR_PYTHON_DIR}/gnuradio/sparsdr
    COMPONENT pythonapi)
