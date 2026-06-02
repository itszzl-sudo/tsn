#ifndef TSN_RUNTIME_H
#define TSN_RUNTIME_H

#include <stdint.h>

// NaN-boxing constants
#define UNDEFINED 0x7FFF800000000001ULL
#define NULL_VAL  0x7FFF800000000002ULL
#define TRUE      0x7FFF000000000001ULL
#define FALSE     0x7FFF000000000000ULL
#define STRING_TAG 0x7FFC000000000000ULL

static inline uint64_t bits_to_val(uint64_t bits) {
    return *(double*)&bits;
}

static inline uint64_t val_to_bits(double val) {
    return *(uint64_t*)&val;
}

#endif