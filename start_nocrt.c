#include <stdint.h>
#include <windows.h>

__declspec(noinline) void* memcpy(void* dst, const void* src, size_t len) {
    volatile unsigned char* d = (volatile unsigned char*)dst;
    const volatile unsigned char* s = (const volatile unsigned char*)src;
    for (size_t i = 0; i < len; i++) d[i] = s[i];
    return dst;
}

__declspec(noinline) void* memmove(void* dst, const void* src, size_t len) {
    volatile unsigned char* d = (volatile unsigned char*)dst;
    const volatile unsigned char* s = (const volatile unsigned char*)src;
    if (d < s) {
        for (size_t i = 0; i < len; i++) d[i] = s[i];
    } else {
        for (size_t i = len; i > 0; i--) d[i-1] = s[i-1];
    }
    return dst;
}

__declspec(noinline) void* memset(void* dst, int val, size_t len) {
    volatile unsigned char* d = (volatile unsigned char*)dst;
    for (size_t i = 0; i < len; i++) d[i] = (unsigned char)val;
    return dst;
}

#define UNDEFINED 0x7FFF800000000001ULL
#define NULL_VAL  0x7FFF800000000002ULL
#define TRUE_VAL  0x7FFF000000000001ULL
#define FALSE_VAL 0x7FFF000000000000ULL
#define STRING_TAG 0x7FFC000000000000ULL
#define ARRAY_TAG  0x7FFB000000000000ULL
#define OBJECT_TAG 0x7FFA000000000000ULL
#define TAG_MASK   0xFFFF000000000000ULL
#define PTR_MASK   0x0000FFFFFFFFFFFFULL

typedef struct { uint32_t len; uint32_t hash; uint8_t data[]; } JsString;
typedef struct { uint32_t len; uint32_t capacity; uint64_t data[]; } JsArray;
typedef struct { uint64_t key; uint64_t value; } ObjectEntry;
typedef struct { uint32_t size; uint32_t capacity; ObjectEntry entries[]; } JsObject;

static double bits_to_val(uint64_t bits) {
    double result;
    memcpy(&result, &bits, sizeof(double));
    return result;
}

static uint64_t val_to_bits(double val) {
    uint64_t bits;
    memcpy(&bits, &val, sizeof(uint64_t));
    return bits;
}

extern double main(void);

// 定义_fltused符号（MSVC浮点数支持）
#ifdef _WIN64
__declspec(selectany) const double _fltused = 0;
#else
const double _fltused = 0;
#endif

void _start() {
    SetConsoleOutputCP(65001);
    double result = main();
    int exit_code = (int)result;
    ExitProcess(exit_code);
}
