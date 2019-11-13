/* -*- c++ -*- */
/*
 * Copyright 2019 The Regents of the University of California.
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

#ifdef HAVE_CONFIG_H
#include "config.h"
#endif

#include <QApplication>
#include <gnuradio/io_signature.h>
#include "average_waterfall_impl.h"

namespace gr {
  namespace sparsdr {

    average_waterfall::sptr
    average_waterfall::make(std::size_t max_history, QWidget* parent)
    {
      return gnuradio::get_initial_sptr
        (new average_waterfall_impl(max_history, parent));
    }

    /*
     * The private constructor
     */
    average_waterfall_impl::average_waterfall_impl(std::size_t max_history, QWidget* parent)
      : gr::sync_block("average_waterfall",
            // One input of SparSDR compressed samples
            // (this matches the UHD complex short sample size)
              gr::io_signature::make(1, 1, sizeof(std::uint32_t)),
              gr::io_signature::make(0, 0, 0)),
        d_average_model(max_history),
        d_parent(parent),
        d_main_gui(nullptr)
    {
        // Required now for Qt; argc must be greater than 0 and argv
        // must have at least one valid character. Must be valid through
        // life of the qApplication:
        // http://harmattan-dev.nokia.com/docs/library/html/qt4/qapplication.html
        d_argc = 1;
        d_argv = new char;
        d_argv[0] = '\0';

        initialize();
    }

    /*
     * Our virtual destructor.
     */
    average_waterfall_impl::~average_waterfall_impl()
    {
        delete d_argv;
    }

    void
    average_waterfall_impl::initialize() {
        if (qApp != NULL) {
            d_qApplication = qApp;
        } else {
            d_qApplication = new QApplication(d_argc, &d_argv);
        }
        d_main_gui = new AverageWaterfallView(d_parent);
        d_main_gui->setModel(&d_average_model);
    }

    QWidget*
    average_waterfall_impl::qwidget() {
        return d_main_gui;
    }

    PyObject*
    average_waterfall_impl::pyqwidget() {
        PyObject* w = PyLong_FromVoidPtr(d_main_gui);
        PyObject* retarg = Py_BuildValue("N", w);
        return retarg;
    }

    void average_waterfall_impl::exec_() {
        d_qApplication->exec();
    }

    int
    average_waterfall_impl::work(int noutput_items,
        gr_vector_const_void_star &input_items,
        gr_vector_void_star &output_items)
    {
        // One sample is really 8 bytes
        const auto nsamples = noutput_items / 2;
        const std::uint8_t* in = static_cast<const std::uint8_t*>(input_items[0]);

        // Do <+signal processing+>
        for (int i = 0; i < nsamples; i++) {
            const std::uint8_t* sample_bytes = in + 8 * i;
            // Parse sample
            const auto fft_index = static_cast<std::uint16_t>(sample_bytes[0])
                | static_cast<std::uint16_t>(sample_bytes[1]) << 8;
            const auto is_average = ((fft_index >> 15) & 1) == 1;
            if (is_average) {
                const auto index = (fft_index >> 4) & 0x7ff;

                // Magnitude is in two 2-byte chunks. Bytes within each chunk are little endian,
                // but the more significant chunk is first.
                const auto mag_more_significant = static_cast<std::uint16_t>(sample_bytes[4])
                    | static_cast<std::uint16_t>(sample_bytes[5]) << 8;
                const auto mag_less_significant = static_cast<std::uint16_t>(sample_bytes[6])
                    | static_cast<std::uint16_t>(sample_bytes[7]) << 8;
                const auto magnitude = static_cast<std::uint32_t>(mag_more_significant)
                    << 16 | static_cast<std::uint32_t>(mag_less_significant);

                // Add this average to the model
                d_average_model.store_sample(index, magnitude);
            }
        }

        // Update the GUI with the new samples
        d_main_gui->update();

        // Tell runtime system how many items were processed
        return nsamples * 2;
    }

  } /* namespace sparsdr */
} /* namespace gr */
