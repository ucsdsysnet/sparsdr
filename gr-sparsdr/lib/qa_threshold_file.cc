/* -*- c++ -*- */
/*
 * Copyright 2022 The Regents of the University of California.
 *
 * This is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 3, or (at your option)
 * any later version.
 *
 * This software is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this software; see the file COPYING.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street,
 * Boston, MA 02110-1301, USA.
 */

#define BOOST_TEST_MODULE sparsdr
#include "threshold_file.h"
#include <boost/test/included/unit_test.hpp>
#include <sstream>

namespace gr {
namespace sparsdr {

namespace {

void check_simple_file(const threshold_file& file)
{
    BOOST_REQUIRE_EQUAL(file.gain, 31);
    BOOST_REQUIRE_EQUAL(file.shift_amount, 5);
    BOOST_REQUIRE_EQUAL(file.thresholds.size(), 1024);

    BOOST_REQUIRE_EQUAL(file.thresholds.at(512), 55);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(513), 55);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(514), 54);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(515), 53);

    BOOST_REQUIRE_EQUAL(file.thresholds.at(1020), 2);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(1021), 51);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(1022), 55);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(1023), 59);

    BOOST_REQUIRE_EQUAL(file.thresholds.at(0), 63);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(1), 63);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(2), 63);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(3), 63);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(4), 63);

    BOOST_REQUIRE_EQUAL(file.thresholds.at(505), 60);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(506), 61);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(507), 62);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(508), 62);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(509), 63);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(510), 64);
    BOOST_REQUIRE_EQUAL(file.thresholds.at(511), 65);
}

} // namespace

BOOST_AUTO_TEST_SUITE(threshold_file_test);

BOOST_AUTO_TEST_CASE(empty_file)
{
    std::stringstream empty;

    try {
        threshold_file::from_stream(empty, 1024);
        BOOST_FAIL("No exception thrown on empty file");
    } catch (...) {
        // OK
    }
}

BOOST_AUTO_TEST_CASE(simple_file_newline_at_end)
{
    const threshold_file file =
        threshold_file::from_file("./thresholds_newline_at_end.txt", 1024);
    check_simple_file(file);
}
BOOST_AUTO_TEST_CASE(simple_file_no_newline_at_end)
{
    const threshold_file file =
        threshold_file::from_file("./thresholds_no_newline_at_end.txt", 1024);
    check_simple_file(file);
}

BOOST_AUTO_TEST_SUITE_END();

} /* namespace sparsdr */
} /* namespace gr */
