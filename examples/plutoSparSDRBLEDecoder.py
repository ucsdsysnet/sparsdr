#!/usr/bin/env python3
# -*- coding: utf-8 -*-

#
# SPDX-License-Identifier: GPL-3.0
#
# GNU Radio Python Flow Graph
# Title: plutoSparSDR
# GNU Radio version: v3.8.5.0-5-g982205bd

from distutils.version import StrictVersion

if __name__ == '__main__':
    import ctypes
    import sys
    if sys.platform.startswith('linux'):
        try:
            x11 = ctypes.cdll.LoadLibrary('libX11.so')
            x11.XInitThreads()
        except:
            print("Warning: failed to XInitThreads()")

from PyQt5 import Qt
from gnuradio import blocks
from gnuradio import qtgui
from gnuradio.filter import firdes
import sip
from gnuradio import gr
import sys
import signal
from argparse import ArgumentParser
from gnuradio.eng_arg import eng_float, intx
from gnuradio import eng_notation
import sparsdr
import distutils.spawn
import argparse

import gr_bluetooth
import iio

from gnuradio import qtgui

class plutoSparSDR(gr.top_block, Qt.QWidget):

    def __init__(self, args):
        gr.top_block.__init__(self, "plutoSparSDR")
        Qt.QWidget.__init__(self)
        self.setWindowTitle("plutoSparSDR")
        qtgui.util.check_set_qss()
        try:
            self.setWindowIcon(Qt.QIcon.fromTheme('gnuradio-grc'))
        except:
            pass
        self.top_scroll_layout = Qt.QVBoxLayout()
        self.setLayout(self.top_scroll_layout)
        self.top_scroll = Qt.QScrollArea()
        self.top_scroll.setFrameStyle(Qt.QFrame.NoFrame)
        self.top_scroll_layout.addWidget(self.top_scroll)
        self.top_scroll.setWidgetResizable(True)
        self.top_widget = Qt.QWidget()
        self.top_scroll.setWidget(self.top_widget)
        self.top_layout = Qt.QVBoxLayout(self.top_widget)
        self.top_grid_layout = Qt.QGridLayout()
        self.top_layout.addLayout(self.top_grid_layout)

        self.settings = Qt.QSettings("GNU Radio", "plutoSparSDR")

        try:
            if StrictVersion(Qt.qVersion()) < StrictVersion("5.0.0"):
                self.restoreGeometry(self.settings.value("geometry").toByteArray())
            else:
                self.restoreGeometry(self.settings.value("geometry"))
        except:
            pass

        ##################################################
        # Variables
        ##################################################
        variable_sparsdr_reconstruct_0_bands = sparsdr.band_spec_vector()
        variable_sparsdr_reconstruct_0_bands.push_back(sparsdr.band_spec(0.0, 1024))
        self.variable_sparsdr_reconstruct_0 = variable_sparsdr_reconstruct_0 = sparsdr.reconstruct(bands=variable_sparsdr_reconstruct_0_bands, 
                                                    reconstruct_path=distutils.spawn.find_executable(args.execpath), 
                                                    sample_format='Pluto v2', zero_gaps=False, compression_fft_size=1024)
        self.samp_rate = samp_rate = 61440000

        ##################################################
        # Blocks
        ##################################################
        self.sparsdr_compressing_pluto_source_0 = sparsdr.compressing_pluto_source('ip:192.168.2.1', 1024 * 1024)
        self.sparsdr_compressing_pluto_source_0.set_frequency(2414000000)
        self.sparsdr_compressing_pluto_source_0.set_gain(42)

        self.sparsdr_compressing_pluto_source_0.stop_all()
        self.sparsdr_compressing_pluto_source_0.set_shift_amount(7)
        self.sparsdr_compressing_pluto_source_0.set_fft_size(1024)
        self.sparsdr_compressing_pluto_source_0.load_rounded_hann_window(1024)
        self.sparsdr_compressing_pluto_source_0.set_bin_spec('184..215:1000,808..839:1000')
        self.sparsdr_compressing_pluto_source_0.set_average_interval(2 ** 20)
        self.sparsdr_compressing_pluto_source_0.start_all()

        self.qtgui_sink_x_0 = qtgui.sink_c(
            8192, #fftsize
            firdes.WIN_BLACKMAN_hARRIS, #wintype
            2414e6, #fc
            samp_rate, #bw
            "", #name
            True, #plotfreq
            True, #plotwaterfall
            True, #plottime
            True #plotconst
        )
        self.qtgui_sink_x_0.set_update_time(1.0/100)
        self._qtgui_sink_x_0_win = sip.wrapinstance(self.qtgui_sink_x_0.pyqwidget(), Qt.QWidget)
        self.qtgui_sink_x_0.enable_rf_freq(True)

        self.top_layout.addWidget(self._qtgui_sink_x_0_win)

        # BLE Decoder
        self.bluetooth_gr_bluetooth_multi_sniffer_0 = gr_bluetooth.multi_sniffer(samp_rate, 2414000000, squelch_threshold = 0.0, tun = True)

        self.blocks_multiply_const_vxx_0 = blocks.multiply_const_cc(1024)


        ##################################################
        # Connections
        ##################################################
        self.connect((self.sparsdr_compressing_pluto_source_0, 0), (self.variable_sparsdr_reconstruct_0, 0))
        self.connect((self.variable_sparsdr_reconstruct_0, 0), (self.blocks_multiply_const_vxx_0, 0))
        self.connect((self.blocks_multiply_const_vxx_0, 0), (self.qtgui_sink_x_0, 0))
        self.connect((self.blocks_multiply_const_vxx_0, 0), (self.bluetooth_gr_bluetooth_multi_sniffer_0, 0))


    def closeEvent(self, event):
        self.settings = Qt.QSettings("GNU Radio", "plutoSparSDR")
        self.settings.setValue("geometry", self.saveGeometry())
        event.accept()

    def get_variable_sparsdr_reconstruct_0(self):
        return self.variable_sparsdr_reconstruct_0

    def set_variable_sparsdr_reconstruct_0(self, variable_sparsdr_reconstruct_0):
        self.variable_sparsdr_reconstruct_0 = variable_sparsdr_reconstruct_0

    def get_samp_rate(self):
        return self.samp_rate

    def set_samp_rate(self, samp_rate):
        self.samp_rate = samp_rate
        self.qtgui_sink_x_0.set_frequency_range(0, self.samp_rate)


def init_argparse() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        usage="%(prog)s <execpath>",
        description="Listen to BLE advertising channel 37 (2402 MHz) and channel 38 (2426 MHz) using Pluto-SDR with SparSDRv2"
    )
    parser.add_argument('execpath', type=str, 
             help='Path to reconstruct executable (required)')
    return parser


def main(top_block_cls=plutoSparSDR, options=None):

    parser = init_argparse()
    args = parser.parse_args()

    if StrictVersion("4.5.0") <= StrictVersion(Qt.qVersion()) < StrictVersion("5.0.0"):
        style = gr.prefs().get_string('qtgui', 'style', 'raster')
        Qt.QApplication.setGraphicsSystem(style)
    qapp = Qt.QApplication(sys.argv)

    tb = top_block_cls(args)

    tb.start()

    tb.show()

    def sig_handler(sig=None, frame=None):
        Qt.QApplication.quit()

    signal.signal(signal.SIGINT, sig_handler)
    signal.signal(signal.SIGTERM, sig_handler)

    timer = Qt.QTimer()
    timer.start(500)
    timer.timeout.connect(lambda: None)

    def quitting():
        tb.stop()
        tb.wait()

    qapp.aboutToQuit.connect(quitting)
    qapp.exec_()

if __name__ == '__main__':
    main()
