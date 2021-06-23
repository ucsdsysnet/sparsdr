#ifndef INCLUDED_SPARSDR_MASK_RANGE_H
#define INCLUDED_SPARSDR_MASK_RANGE_H

#include <cstdint>

namespace gr {
namespace sparsdr {

/*!
 * An optional range of bins to mask
 *
 * If start and end are both 0, this mask range does not mask any bins.
 */
struct mask_range {
public:
    /*! \brief Start bin, inclusive */
    uint16_t start;
    /*! \brief End bin, exclusive */
    uint16_t end;

    /*! \brief Returns a default mask range that does not mask any bins */
    inline mask_range() : start(0), end(0) {}

    /*! \brief Creates a mask range with start and end bin numbers */
    inline mask_range(uint16_t start, uint16_t end) : start(start), end(end) {}
};

} // namespace sparsdr
} // namespace gr

#endif
