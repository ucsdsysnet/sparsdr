# Copyright 2011 Free Software Foundation, Inc.
#
# This file is part of GNU Radio
#
# GNU Radio is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 3, or (at your option)
# any later version.
#
# GNU Radio is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with GNU Radio; see the file COPYING.  If not, write to
# the Free Software Foundation, Inc., 51 Franklin Street,
# Boston, MA 02110-1301, USA.

include(GrPython)

GR_PYTHON_INSTALL(
    PROGRAMS
    DESTINATION bin
)

# C++

# sparsdr_real_time_receive

add_executable(sparsdr_real_time_receive
    sparsdr_real_time_receive.cc
)
target_include_directories(sparsdr_real_time_receive
    PRIVATE
    ${UHD_INCLUDE_DIRS}
)
target_link_libraries(sparsdr_real_time_receive
    gnuradio-sparsdr
    ${UHD_LIBRARIES}
)
install(
    TARGETS sparsdr_real_time_receive
    DESTINATION bin
)

find_package(gr_bluetooth)

if(GR_BLUETOOTH_FOUND)
    # sparsdr_bluetooth_sniffer
    add_executable(sparsdr_bluetooth_sniffer
        sparsdr_bluetooth_sniffer.cc
    )
    target_include_directories(sparsdr_bluetooth_sniffer
        PRIVATE
        ${GR_BLUETOOTH_INCLUDE_DIRS}
    )
    target_link_libraries(sparsdr_bluetooth_sniffer
        gnuradio-sparsdr
        ${GR_BLUETOOTH_LIBRARIES}
    )
    install(
        TARGETS sparsdr_bluetooth_sniffer
        DESTINATION bin
    )

    # sparsdr_receive_sniffer
    add_executable(sparsdr_receive_sniffer
        sparsdr_receive_sniffer.cc
    )
    target_include_directories(sparsdr_receive_sniffer
        PRIVATE
        ${GR_BLUETOOTH_INCLUDE_DIRS}
    )
    target_link_libraries(sparsdr_receive_sniffer
        gnuradio-sparsdr
        ${GR_BLUETOOTH_LIBRARIES}
        ${UHD_LIBRARIES}
    )
    install(
        TARGETS sparsdr_receive_sniffer
        DESTINATION bin
    )

    # sparsdr_receive_sniffer_time_logging
    add_executable(sparsdr_receive_sniffer_time_logging
        sparsdr_receive_sniffer_time_logging.cc
    )
    target_include_directories(sparsdr_receive_sniffer_time_logging
        PRIVATE
        ${GR_BLUETOOTH_INCLUDE_DIRS}
    )
    target_link_libraries(sparsdr_receive_sniffer_time_logging
        gnuradio-sparsdr
        ${GR_BLUETOOTH_LIBRARIES}
        ${UHD_LIBRARIES}
    )
    install(
        TARGETS sparsdr_receive_sniffer_time_logging
        DESTINATION bin
    )
endif()
