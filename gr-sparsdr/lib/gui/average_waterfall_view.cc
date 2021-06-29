#include "average_waterfall_view.h"

#include <QDebug>
#include <QPainter>

namespace gr {
namespace sparsdr {

AverageWaterfallView::AverageWaterfallView(QWidget* parent)
    : QWidget(parent), _model(nullptr)
{
}

void AverageWaterfallView::paintEvent(QPaintEvent*)
{
    if (!_model) {
        return;
    }
    const auto max_average = _model->max();
    if (max_average == 0) {
        // Nothing to draw, just fill the widget with black
        QPainter painter(this);
        painter.fillRect(rect(), Qt::black);
    } else {
        // Create a pixmap where each pixel is one average value
        // and each row is one time unit
        QPixmap waterfall(2048, static_cast<int>(_model->size()));
        QPainter waterfallPainter(&waterfall);
        // Draw each value
        for (std::size_t y = 0; y < _model->size(); y++) {
            const auto averages = _model->averages(y);
            for (int x = 0; x < 2048; x++) {
                // Set color: Brightness is proportional to average value
                const auto average_value = averages[x];
                const auto scaled_brightness =
                    static_cast<double>(average_value) / static_cast<double>(max_average);
                const auto color = QColor::fromHsvF(0.0, 0.0, scaled_brightness);
                waterfallPainter.setPen(color);
                waterfallPainter.drawPoint(x, static_cast<int>(y));
            }
        }

        // Draw the waterfall into the widget, scaled
        QPainter painter(this);
        painter.drawPixmap(rect(), waterfall);
    }
}

} // namespace sparsdr
} // namespace gr
