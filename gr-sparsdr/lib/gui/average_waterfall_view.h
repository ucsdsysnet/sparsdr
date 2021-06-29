#ifndef AVERAGEWATERFALLVIEW_H
#define AVERAGEWATERFALLVIEW_H

#include "average_model.h"
#include <QWidget>

namespace gr {
namespace sparsdr {

class AverageWaterfallView : public QWidget
{
    Q_OBJECT
public:
    explicit AverageWaterfallView(QWidget* parent = nullptr);

    /**
     * @brief setModel sets the model to use to get averages
     * @param model
     */
    inline void setModel(AverageModel* model) { _model = model; }

    virtual void paintEvent(QPaintEvent* event) override;

signals:

public slots:

private:
    /**
     * @brief The model used to get averages
     */
    AverageModel* _model;
};

} // namespace sparsdr
} // namespace gr
#endif // AVERAGEWATERFALLVIEW_H
