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


#ifndef INCLUDED_SPARSDR_AVERAGE_WATERFALL_H
#define INCLUDED_SPARSDR_AVERAGE_WATERFALL_H

#include <gnuradio/sync_block.h>
#include <Python.h>
#include <sparsdr/api.h>
#include <QWidget>

namespace gr {
namespace sparsdr {

/*!
 * \brief Displays a waterfall view showing average values from a SparSDR
 * receiver
 * \ingroup sparsdr
 *
 */
class SPARSDR_API average_waterfall : virtual public gr::sync_block
{
public:
    typedef boost::shared_ptr<average_waterfall> sptr;

    /*!
     * \brief Return a shared_ptr to a new instance of sparsdr::average_waterfall.
     *
     * To avoid accidental use of raw pointers, sparsdr::average_waterfall's
     * constructor is in a private implementation
     * class. sparsdr::average_waterfall::make is the public interface for
     * creating new instances.
     */
    static sptr make(std::size_t max_history = 2048, QWidget* parent = nullptr);

    virtual void exec_() = 0;
    virtual QWidget* qwidget() = 0;
    virtual PyObject* pyqwidget() = 0;


    QApplication* d_qApplication;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_AVERAGE_WATERFALL_H */
