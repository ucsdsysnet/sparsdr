#ifndef INCLUDED_SPARSDR_PRIVATE_REGISTERS_H
#define INCLUDED_SPARSDR_PRIVATE_REGISTERS_H

namespace gr {
  namespace sparsdr {
    namespace detail {
      namespace registers {
        /** Scaling (what is this?) */
        static const uint8_t SCALING = 10;
        /** Per-bin threshold set command */
        static const uint8_t THRESHOLD = 11;
        /** Per-bin mask set command */
        static const uint8_t MASK = 12;
        /** Average weight */
        static const uint8_t AVG_WEIGHT = 13;
        /** Average interval */
        static const uint8_t AVG_INTERVAL = 14;
        /** Enable FFT sample sending */
        static const uint8_t FFT_SEND = 15;
        /** Enable average sample sending */
        static const uint8_t AVG_SEND = 16;
        /** Enable FFT */
        static const uint8_t RUN_FFT = 17;
        /** Register to enable/disable compression */
        static const uint8_t ENABLE_COMPRESSION = 19;
        /** FFT size */
        static const uint8_t FFT_SIZE = 20;
      }
    }
  }
}

#endif
