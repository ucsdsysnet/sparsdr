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

#ifndef INCLUDED_SPARSDR_AVERAGE_WATERFALL_IMPL_H
#define INCLUDED_SPARSDR_AVERAGE_WATERFALL_IMPL_H

#include "average_waterfall_view.h"
#include "stream_average_model.h"
#include <sparsdr/average_waterfall.h>

namespace gr {
namespace sparsdr {

class average_waterfall_impl : public average_waterfall
{
private:
    /** Stores averages for the GUI */
    stream_average_model d_average_model;

    int d_argc;
    char* d_argv;
    /** Parent of waterfall GUI */
    QWidget* d_parent;
    /** Actual waterfall GUI */
    AverageWaterfallView* d_main_gui;

    void buildwindow();
    void initialize();

public:
    average_waterfall_impl(std::size_t max_history, QWidget* parent);
    ~average_waterfall_impl();

    // Where all the action really happens
    int work(int noutput_items,
             gr_vector_const_void_star& input_items,
             gr_vector_void_star& output_items);

    virtual void exec_() override;
    virtual QWidget* qwidget() override;
    virtual PyObject* pyqwidget() override;
};

} // namespace sparsdr
} // namespace gr

#endif /* INCLUDED_SPARSDR_AVERAGE_WATERFALL_IMPL_H */
