#ifndef SPARSDR_OUTPUT_CONTEXT_H
#define SPARSDR_OUTPUT_CONTEXT_H

#include <condition_variable>
#include <complex>
#include <mutex>
#include <queue>

namespace gr {
namespace sparsdr {

/** A value used with the reconstruct output callback as a context */
struct output_context {
    /** Mutex used to protect queue */
    std::mutex mutex;
    /**
     * Condition variable used to notify the work thread when samples are
     * available
     */
    std::condition_variable cv;
    /** Queue of samples produced by the reconstruction library */
    std::queue<std::complex<float>> queue;
};

} // namespace sparsdr
} // namespace gr

#endif
