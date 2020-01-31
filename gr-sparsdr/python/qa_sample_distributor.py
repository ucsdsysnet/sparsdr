#!/usr/bin/env python
# -*- coding: utf-8 -*-
#
# Copyright 2020 The Regents of the University of California.
#
# This is free software; you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation; either version 3, or (at your option)
# any later version.
#
# This software is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this software; see the file COPYING.  If not, write to
# the Free Software Foundation, Inc., 51 Franklin Street,
# Boston, MA 02110-1301, USA.
#

from gnuradio import gr, gr_unittest
from gnuradio import blocks
# import sparsdr_swig as sparsdr
import sparsdr

class qa_sample_distributor(gr_unittest.TestCase):

    def setUp(self):
        self.tb = gr.top_block()

    def tearDown(self):
        self.tb = None

    def test_no_inputs_no_outputs(self):
        # Just add a sample distributor
        head = blocks.head(gr.sizeof_gr_complex, 0)
        null_source = blocks.null_source(gr.sizeof_gr_complex)
        null_sink = blocks.null_sink(gr.sizeof_gr_complex)
        sample_distributor = sparsdr.sample_distributor(gr.sizeof_gr_complex)
        # self.tb.connect(sample_distributor)
        # self.tb.connect(null_source, sample_distributor)
        # self.tb.connect(sample_distributor, null_sink)
        self.tb.connect(null_source, head, sample_distributor, null_sink)
        self.tb.run()
        # Nothing to check


if __name__ == '__main__':
    gr_unittest.run(qa_sample_distributor, "qa_sample_distributor.xml")
