#include <stdint.h>
#include <windows.h>

__declspec(noinline) static void js_memcpy(void* dst, const void* src, size_t len) {
    volatile unsigned char* d = (volatile unsigned char*)dst;
    const volatile unsigned char* s = (const volatile unsigned char*)src;
    for (size_t i = 0; i < len; i++) d[i] = s[i];
}
#define memcpy js_memcpy

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

static void* js_malloc(size_t size) {
    return HeapAlloc(GetProcessHeap(), 0, size);
}

static void* js_realloc(void* ptr, size_t size) {
    return HeapReAlloc(GetProcessHeap(), 0, ptr, size);
}

double js_string_new(const char* data, uint32_t len);
double js_string_from_static(const char* data);

static void write_str(const char* s) {
    int len = 0;
    while (s[len]) len++;
    WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), s, len, NULL, NULL);
}

static void write_buf(const char* buf, int len) {
    WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), buf, len, NULL, NULL);
}

static int int64_to_str(int64_t num, char* buf) {
    if (num == 0) {
        buf[0] = '0';
        return 1;
    }
    
    int neg = 0;
    uint64_t n;
    if (num < 0) {
        neg = 1;
        n = (uint64_t)(-(int64_t)num);
    } else {
        n = (uint64_t)num;
    }
    
    char tmp[32];
    int i = 0;
    while (n > 0) {
        tmp[i++] = '0' + (n % 10);
        n /= 10;
    }
    
    int len = 0;
    if (neg) buf[len++] = '-';
    while (i > 0) {
        buf[len++] = tmp[--i];
    }
    return len;
}

static int int_to_str(int num, char* buf) {
    if (num == 0) {
        buf[0] = '0';
        return 1;
    }
    
    // Use unsigned to avoid INT_MIN overflow: -INT_MIN is UB for signed int
    int neg = 0;
    unsigned int n;
    if (num < 0) {
        neg = 1;
        n = (unsigned int)(-(long long)num);
    } else {
        n = (unsigned int)num;
    }
    
    char tmp[32];
    int i = 0;
    while (n > 0) {
        tmp[i++] = '0' + (n % 10);
        n /= 10;
    }
    
    int len = 0;
    if (neg) buf[len++] = '-';
    while (i > 0) {
        buf[len++] = tmp[--i];
    }
    return len;
}

static void print_val(double val) {
    char buf[256];
    uint64_t bits = val_to_bits(val);
    uint64_t tag = bits & TAG_MASK;
    
    int len = 0;
    
    if (bits == UNDEFINED) {
        write_str("undefined");
    }
    else if (bits == NULL_VAL) {
        write_str("null");
    }
    else if (bits == TRUE_VAL) {
        write_str("true");
    }
    else if (bits == FALSE_VAL) {
        write_str("false");
    }
    else if (tag == STRING_TAG) {
        JsString* s = (JsString*)(bits & PTR_MASK);
        if (s) {
            write_buf((const char*)s->data, s->len);
        } else {
            write_str("null");
        }
    }
    else if (tag == ARRAY_TAG) {
        JsArray* arr = (JsArray*)(bits & PTR_MASK);
        if (arr) {
            buf[len++] = '[';
            write_buf(buf, len);
            len = 0;
            

            for (uint32_t i = 0; i < arr->len; i++) {
                if (i > 0) {
                    write_str(", ");
                }
                
                uint64_t elem_bits = arr->data[i];
                if ((elem_bits & TAG_MASK) == STRING_TAG) {
                    JsString* s = (JsString*)(elem_bits & PTR_MASK);
                    if (s) {
                        write_buf((const char*)s->data, s->len);
                    } else {
                        write_str("null");
                    }
                }
                else if (elem_bits == UNDEFINED) {
                    write_str("undefined");
                } else if (elem_bits == NULL_VAL) {
                    write_str("null");
                } else if (elem_bits == TRUE_VAL) {
                    write_str("true");
                } else if (elem_bits == FALSE_VAL) {
                    write_str("false");
                } else {
                    double elem_val = bits_to_val(elem_bits);
                    int64_t num = (int64_t)elem_val;
                    len = int64_to_str(num, buf);
                    write_buf(buf, len);
                    len = 0;
                }
            }
            write_str("]");
        } else {
            write_str("null");
        }
    }
    else if (tag == OBJECT_TAG) {
        JsObject* obj = (JsObject*)(bits & PTR_MASK);
        if (obj) {
            write_str("{");
            
            for (uint32_t i = 0; i < obj->size; i++) {
                if (i > 0) {
                    write_str(", ");
                }
                
                write_str("\"key\": ");
                
                uint64_t val_bits = obj->entries[i].value;
                if (val_bits == UNDEFINED) {
                    write_str("undefined");
                } else if (val_bits == NULL_VAL) {
                    write_str("null");
                } else if (val_bits == TRUE_VAL) {
                    write_str("true");
                } else if (val_bits == FALSE_VAL) {
                    write_str("false");
                } else {
                    double val = bits_to_val(val_bits);
                    int64_t num = (int64_t)val;
                    len = int64_to_str(num, buf);
                    write_buf(buf, len);
                    len = 0;
                }
            }
            write_str("}");
        } else {
            write_str("null");
        }
    }
    else {
        int64_t num = (int64_t)val;
        double frac = val - (double)num;
        if (frac < 0) frac = -frac;
        if (frac > 1e-9 && frac < 1.0 - 1e-9) {
            len = int64_to_str(num, buf);
            buf[len++] = '.';
            int frac_part = (int)(frac * 1000000);
            char frac_buf[16];
            int fl = int_to_str(frac_part, frac_buf);
            for (int i = 0; i < fl; i++) buf[len++] = frac_buf[i];
            write_buf(buf, len);
        } else {
            len = int64_to_str(num, buf);
            write_buf(buf, len);
        }
    }
}

double js_print(double val) {
    print_val(val);
    return val;
}

double js_print_space(void) {
    write_str(" ");
    return bits_to_val(UNDEFINED);
}

double js_print_newline(void) {
    write_str("\n");
    return bits_to_val(UNDEFINED);
}

double js_array_new(double capacity_d) {
    uint32_t capacity = (uint32_t)capacity_d;

    if (capacity == 0) capacity = 8;
    JsArray* arr = (JsArray*)js_malloc(sizeof(JsArray) + capacity * 8);
    if (!arr) return bits_to_val(UNDEFINED);
    arr->len = 0;
    arr->capacity = capacity;
    return bits_to_val(ARRAY_TAG | (uint64_t)arr);
}

double js_array_push(double arr_val, double value) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return arr_val;
    
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr) return arr_val;
    
    if (arr->len >= arr->capacity) {
        uint32_t new_cap = arr->capacity * 2;
        JsArray* new_arr = (JsArray*)js_realloc(arr, sizeof(JsArray) + new_cap * 8);
        if (!new_arr) return arr_val;
        new_arr->capacity = new_cap;
        arr = new_arr;
    }
    
    arr->data[arr->len] = val_to_bits(value);
    arr->len++;
    return bits_to_val(ARRAY_TAG | (uint64_t)arr);
}

double js_array_get(double arr_val, double idx_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(UNDEFINED);
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    int idx = (int)idx_val;
    if (!arr || idx < 0 || (uint32_t)idx >= arr->len) return bits_to_val(UNDEFINED);
    return bits_to_val(arr->data[idx]);
}

double js_array_set(double arr_val, double idx_val, double value) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return value;
    
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    uint32_t idx = (uint32_t)idx_val;
    
    if (!arr) return value;
    
    if (idx >= arr->len) {
        while (arr->len <= idx) {
            if (arr->len >= arr->capacity) {
                uint32_t new_cap = arr->capacity * 2;
                JsArray* new_arr = (JsArray*)js_realloc(arr, sizeof(JsArray) + new_cap * 8);
                if (!new_arr) return value;
                new_arr->capacity = new_cap;
                arr = new_arr;
            }
            arr->data[arr->len++] = UNDEFINED;
        }
    }
    
    arr->data[idx] = val_to_bits(value);
    return value;
}

double js_object_new() {
    JsObject* obj = (JsObject*)js_malloc(sizeof(JsObject) + 8 * sizeof(ObjectEntry));
    if (!obj) return bits_to_val(UNDEFINED);
    obj->size = 0;
    obj->capacity = 8;
    return bits_to_val(OBJECT_TAG | (uint64_t)obj);
}

double js_object_get(double obj_val, double key_val) {
    uint64_t obj_bits = val_to_bits(obj_val);
    uint64_t key_bits = val_to_bits(key_val);
    
    if ((obj_bits & TAG_MASK) != OBJECT_TAG) return bits_to_val(UNDEFINED);
    
    JsObject* obj = (JsObject*)(obj_bits & PTR_MASK);
    if (!obj) return bits_to_val(UNDEFINED);
    
    for (uint32_t i = 0; i < obj->size; i++) {
        if (obj->entries[i].key == key_bits) {
            return bits_to_val(obj->entries[i].value);
        }
    }
    
    return bits_to_val(UNDEFINED);
}

double js_object_set(double obj_val, double key_val, double value_val) {
    uint64_t obj_bits = val_to_bits(obj_val);
    uint64_t key_bits = val_to_bits(key_val);
    uint64_t value_bits = val_to_bits(value_val);
    
    if ((obj_bits & TAG_MASK) != OBJECT_TAG) return value_val;
    
    JsObject* obj = (JsObject*)(obj_bits & PTR_MASK);
    if (!obj) return value_val;
    
    for (uint32_t i = 0; i < obj->size; i++) {
        if (obj->entries[i].key == key_bits) {
            obj->entries[i].value = value_bits;
            return value_val;
        }
    }
    
    if (obj->size >= obj->capacity) {
        uint32_t new_cap = obj->capacity * 2;
        JsObject* new_obj = (JsObject*)js_realloc(obj, sizeof(JsObject) + new_cap * sizeof(ObjectEntry));
        if (!new_obj) return value_val;
        new_obj->capacity = new_cap;
        obj = new_obj;
    }
    
    obj->entries[obj->size].key = key_bits;
    obj->entries[obj->size].value = value_bits;
    obj->size++;
    
    return value_val;
}

double js_typeof(double val) {
    uint64_t bits = val_to_bits(val);
    uint64_t tag = bits & TAG_MASK;
    
    static char* type_undefined = "undefined";
    static char* type_object = "object";
    static char* type_boolean = "boolean";
    static char* type_string = "string";
    static char* type_number = "number";
    
    if (bits == UNDEFINED) return js_string_from_static(type_undefined);
    if (bits == NULL_VAL) return js_string_from_static(type_object);
    if (bits == TRUE_VAL || bits == FALSE_VAL) return js_string_from_static(type_boolean);
    if (tag == STRING_TAG) return js_string_from_static(type_string);
    if (tag == ARRAY_TAG) return js_string_from_static(type_object);
    if (tag == OBJECT_TAG) return js_string_from_static(type_object);
    return js_string_from_static(type_number);
}

double js_string_new(const char* data, uint32_t len) {
    if (!data) return bits_to_val(UNDEFINED);
    
    JsString* str = (JsString*)js_malloc(sizeof(JsString) + (len > 0 ? len : 1));
    if (!str) return bits_to_val(UNDEFINED);
    
    str->len = len;
    str->hash = 0;
    for (uint32_t i = 0; i < len; i++) {
        str->data[i] = data[i];
        str->hash = str->hash * 31 + data[i];
    }
    
    return bits_to_val(STRING_TAG | (uint64_t)str);
}

double js_string_from_static(const char* data) {
    if (!data) return bits_to_val(UNDEFINED);
    
    int len = 0;
    while (data[len]) len++;
    
    return js_string_new(data, (uint32_t)len);
}

int64_t js_unbox_string(double val) {
    uint64_t bits = val_to_bits(val);
    if ((bits & TAG_MASK) != STRING_TAG) return 0;
    JsString* str = (JsString*)(bits & PTR_MASK);
    if (!str) return 0;
    return (int64_t)str->data;
}

double js_box_string(int64_t c_str) {
    if (!c_str) return bits_to_val(UNDEFINED);
    const char* data = (const char*)c_str;
    int len = 0;
    while (data[len]) len++;
    return js_string_new(data, (uint32_t)len);
}

double js_string_concat(double a_val, double b_val) {
    uint64_t a_bits = val_to_bits(a_val);
    uint64_t b_bits = val_to_bits(b_val);
    
    if ((a_bits & TAG_MASK) != STRING_TAG || (b_bits & TAG_MASK) != STRING_TAG) {
        return bits_to_val(UNDEFINED);
    }
    
    JsString* a_str = (JsString*)(a_bits & PTR_MASK);
    JsString* b_str = (JsString*)(b_bits & PTR_MASK);
    
    if (!a_str || !b_str) return bits_to_val(UNDEFINED);
    
    uint32_t new_len = a_str->len + b_str->len;
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + new_len);
    if (!result) return bits_to_val(UNDEFINED);
    
    result->len = new_len;
    result->hash = 0;
    
    for (uint32_t i = 0; i < a_str->len; i++) {
        result->data[i] = a_str->data[i];
        result->hash = result->hash * 31 + a_str->data[i];
    }
    
    for (uint32_t i = 0; i < b_str->len; i++) {
        result->data[a_str->len + i] = b_str->data[i];
        result->hash = result->hash * 31 + b_str->data[i];
    }
    
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_number_to_string(double val) {
    char buf[64];
    int len = 0;
    uint64_t bits = val_to_bits(val);
    
    if (bits == UNDEFINED) {
        return js_string_from_static("undefined");
    }
    if (bits == NULL_VAL) {
        return js_string_from_static("null");
    }
    if (bits == TRUE_VAL) {
        return js_string_from_static("true");
    }
    if (bits == FALSE_VAL) {
        return js_string_from_static("false");
    }
    if ((bits & TAG_MASK) == STRING_TAG) {
        return val;
    }
    
    int64_t num = (int64_t)val;
    double frac = val - (double)num;
    if (frac < 0) frac = -frac;
    
    len = int64_to_str(num, buf);
    if (frac > 1e-9 && frac < 1.0 - 1e-9) {
        buf[len++] = '.';
        int frac_part = (int)(frac * 1000000);
        char frac_buf[16];
        int fl = int_to_str(frac_part, frac_buf);
        for (int i = 0; i < fl; i++) buf[len++] = frac_buf[i];
    }
    
    return js_string_new(buf, (uint32_t)len);
}

double js_add(double a, double b) {
    uint64_t a_bits = val_to_bits(a);
    uint64_t b_bits = val_to_bits(b);
    
    int a_is_string = (a_bits & TAG_MASK) == STRING_TAG;
    int b_is_string = (b_bits & TAG_MASK) == STRING_TAG;
    
    if (a_is_string && b_is_string) {
        return js_string_concat(a, b);
    }
    
    if (a_is_string || b_is_string) {
        double a_str = a_is_string ? a : js_number_to_string(a);
        double b_str = b_is_string ? b : js_number_to_string(b);
        return js_string_concat(a_str, b_str);
    }
    
    return a + b;
}
double js_sub(double a, double b) { return a - b; }
double js_mul(double a, double b) { return a * b; }
double js_div(double a, double b) { return b != 0 ? a / b : bits_to_val(UNDEFINED); }
double js_mod(double a, double b) { return b != 0 ? (double)((int64_t)a % (int64_t)b) : bits_to_val(UNDEFINED); }

double js_eq(double a, double b) {
    uint64_t ua, ub;
    memcpy(&ua, &a, 8);
    memcpy(&ub, &b, 8);
    if (ua == ub) return bits_to_val(TRUE_VAL);
    if (a == b) return bits_to_val(TRUE_VAL);
    return bits_to_val(FALSE_VAL);
}
double js_neq(double a, double b) {
    uint64_t ua, ub;
    memcpy(&ua, &a, 8);
    memcpy(&ub, &b, 8);
    if (ua == ub) return bits_to_val(FALSE_VAL);
    if (a == b) return bits_to_val(FALSE_VAL);
    return bits_to_val(TRUE_VAL);
}
int js_is_truthy(double v) {
    uint64_t bits;
    memcpy(&bits, &v, 8);
    if (bits == FALSE_VAL || bits == NULL_VAL || bits == UNDEFINED || bits == 0) return 0;
    if (bits == TRUE_VAL) return 1;
    if ((bits & TAG_MASK) == STRING_TAG) return 1;
    if ((bits & TAG_MASK) == ARRAY_TAG || (bits & TAG_MASK) == OBJECT_TAG) return 1;
    if (v != v) return 0;
    if (v == 0.0 || v == -0.0) return 0;
    return 1;
}
double js_lt(double a, double b) { return bits_to_val(a < b ? TRUE_VAL : FALSE_VAL); }
double js_le(double a, double b) { return bits_to_val(a <= b ? TRUE_VAL : FALSE_VAL); }
double js_gt(double a, double b) { return bits_to_val(a > b ? TRUE_VAL : FALSE_VAL); }
double js_ge(double a, double b) { return bits_to_val(a >= b ? TRUE_VAL : FALSE_VAL); }

// String methods

double js_string_split(double str_val, double sep_val) {
    uint64_t str_bits = val_to_bits(str_val);
    uint64_t sep_bits = val_to_bits(sep_val);
    
    if ((str_bits & TAG_MASK) != STRING_TAG) return bits_to_val(UNDEFINED);
    
    JsString* str = (JsString*)(str_bits & PTR_MASK);
    if (!str) return bits_to_val(UNDEFINED);
    
    // Create result array
    JsArray* result = (JsArray*)js_malloc(sizeof(JsArray) + 8 * 8);
    if (!result) return bits_to_val(UNDEFINED);
    result->len = 0;
    result->capacity = 8;
    
    if ((sep_bits & TAG_MASK) != STRING_TAG) {
        // No separator, return array with original string
        result->data[0] = str_bits;
        result->len = 1;
        return bits_to_val(ARRAY_TAG | (uint64_t)result);
    }
    
    JsString* sep = (JsString*)(sep_bits & PTR_MASK);
    if (!sep || sep->len == 0) {
        // Empty separator, split each character
        for (uint32_t i = 0; i < str->len; i++) {
            JsString* ch = (JsString*)js_malloc(sizeof(JsString) + 1);
            if (!ch) continue;
            ch->len = 1;
            ch->hash = str->data[i];
            ch->data[0] = str->data[i];
            
            if (result->len >= result->capacity) {
                uint32_t new_cap = result->capacity * 2;
                JsArray* new_arr = (JsArray*)js_realloc(result, sizeof(JsArray) + new_cap * 8);
                if (!new_arr) continue;
                new_arr->capacity = new_cap;
                result = new_arr;
            }
            result->data[result->len++] = STRING_TAG | (uint64_t)ch;
        }
        return bits_to_val(ARRAY_TAG | (uint64_t)result);
    }
    
    // Split by separator
    uint32_t start = 0;
    for (uint32_t i = 0; i <= str->len - sep->len; ) {
        int match = 1;
        for (uint32_t j = 0; j < sep->len; j++) {
            if (str->data[i + j] != sep->data[j]) {
                match = 0;
                break;
            }
        }
        
        if (match) {
            // Create substring
            uint32_t len = i - start;
            JsString* substr = (JsString*)js_malloc(sizeof(JsString) + len);
            if (substr) {
                substr->len = len;
                substr->hash = 0;
                for (uint32_t k = 0; k < len; k++) {
                    substr->data[k] = str->data[start + k];
                    substr->hash = substr->hash * 31 + str->data[start + k];
                }
                
                if (result->len >= result->capacity) {
                    uint32_t new_cap = result->capacity * 2;
                    JsArray* new_arr = (JsArray*)js_realloc(result, sizeof(JsArray) + new_cap * 8);
                    if (new_arr) {
                        new_arr->capacity = new_cap;
                        result = new_arr;
                    }
                }
                result->data[result->len++] = STRING_TAG | (uint64_t)substr;
            }
            
            start = i + sep->len;
            i = start;
        } else {
            i++;
        }
    }
    
    // Add remaining part
    uint32_t len = str->len - start;
    JsString* substr = (JsString*)js_malloc(sizeof(JsString) + len);
    if (substr) {
        substr->len = len;
        substr->hash = 0;
        for (uint32_t k = 0; k < len; k++) {
            substr->data[k] = str->data[start + k];
            substr->hash = substr->hash * 31 + str->data[start + k];
        }
        
        if (result->len >= result->capacity) {
            uint32_t new_cap = result->capacity * 2;
            JsArray* new_arr = (JsArray*)js_realloc(result, sizeof(JsArray) + new_cap * 8);
            if (new_arr) {
                new_arr->capacity = new_cap;
                result = new_arr;
            }
        }
        result->data[result->len++] = STRING_TAG | (uint64_t)substr;
    }
    
    return bits_to_val(ARRAY_TAG | (uint64_t)result);
}

double js_string_replace(double str_val, double search_val, double replace_val) {
    uint64_t str_bits = val_to_bits(str_val);
    uint64_t search_bits = val_to_bits(search_val);
    uint64_t replace_bits = val_to_bits(replace_val);
    
    if ((str_bits & TAG_MASK) != STRING_TAG ||
        (search_bits & TAG_MASK) != STRING_TAG ||
        (replace_bits & TAG_MASK) != STRING_TAG) {
        return bits_to_val(UNDEFINED);
    }
    
    JsString* str = (JsString*)(str_bits & PTR_MASK);
    JsString* search = (JsString*)(search_bits & PTR_MASK);
    JsString* replace = (JsString*)(replace_bits & PTR_MASK);
    
    if (!str || !search || !replace) return bits_to_val(UNDEFINED);
    
    // Find first occurrence
    int32_t found_pos = -1;
    for (uint32_t i = 0; i <= str->len - search->len; i++) {
        int match = 1;
        for (uint32_t j = 0; j < search->len; j++) {
            if (str->data[i + j] != search->data[j]) {
                match = 0;
                break;
            }
        }
        if (match) {
            found_pos = i;
            break;
        }
    }
    
    if (found_pos < 0) {
        // Not found, return original string
        return str_val;
    }
    
    // Build result: before + replace + after
    uint32_t new_len = str->len - search->len + replace->len;
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + new_len);
    if (!result) return bits_to_val(UNDEFINED);
    
    result->len = new_len;
    result->hash = 0;
    
    // Copy before
    for (uint32_t i = 0; i < (uint32_t)found_pos; i++) {
        result->data[i] = str->data[i];
        result->hash = result->hash * 31 + str->data[i];
    }
    
    // Copy replace
    for (uint32_t i = 0; i < replace->len; i++) {
        result->data[found_pos + i] = replace->data[i];
        result->hash = result->hash * 31 + replace->data[i];
    }
    
    // Copy after
    uint32_t after_pos = found_pos + search->len;
    for (uint32_t i = 0; i < str->len - after_pos; i++) {
        result->data[found_pos + replace->len + i] = str->data[after_pos + i];
        result->hash = result->hash * 31 + str->data[after_pos + i];
    }
    
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_string_substring(double str_val, double start_val, double end_val) {
    uint64_t str_bits = val_to_bits(str_val);
    
    if ((str_bits & TAG_MASK) != STRING_TAG) return bits_to_val(UNDEFINED);
    
    JsString* str = (JsString*)(str_bits & PTR_MASK);
    if (!str) return bits_to_val(UNDEFINED);
    
    uint32_t start = (uint32_t)start_val;
    uint32_t end = (uint32_t)end_val;
    
    if (start > str->len) start = str->len;
    if (end > str->len) end = str->len;
    if (start > end) {
        uint32_t tmp = start;
        start = end;
        end = tmp;
    }
    
    uint32_t len = end - start;
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + len);
    if (!result) return bits_to_val(UNDEFINED);
    
    result->len = len;
    result->hash = 0;
    for (uint32_t i = 0; i < len; i++) {
        result->data[i] = str->data[start + i];
        result->hash = result->hash * 31 + str->data[start + i];
    }
    
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_string_char_at(double str_val, double idx_val) {
    uint64_t str_bits = val_to_bits(str_val);
    
    if ((str_bits & TAG_MASK) != STRING_TAG) return bits_to_val(UNDEFINED);
    
    JsString* str = (JsString*)(str_bits & PTR_MASK);
    if (!str) return bits_to_val(UNDEFINED);
    
    uint32_t idx = (uint32_t)idx_val;
    if (idx >= str->len) return bits_to_val(UNDEFINED);
    
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + 1);
    if (!result) return bits_to_val(UNDEFINED);
    
    result->len = 1;
    result->hash = str->data[idx];
    result->data[0] = str->data[idx];
    
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_string_length(double str_val) {
    uint64_t str_bits = val_to_bits(str_val);
    
    if ((str_bits & TAG_MASK) != STRING_TAG) return bits_to_val(UNDEFINED);
    
    JsString* str = (JsString*)(str_bits & PTR_MASK);
    if (!str) return bits_to_val(UNDEFINED);
    
    return (double)str->len;
}

// Array methods

double js_array_length(double arr_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(UNDEFINED);
    
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr) return bits_to_val(UNDEFINED);
    
    return (double)arr->len;
}

// ============================================================
// try-catch 异常处理运行时
// ============================================================
// 
// 使用 Windows NtContinue/RtlRestoreContext 实现异常传播，
// 避免 setjmp/longjmp 在 nocrt 模式下的链接问题。
// 
// 实际实现策略：基于全局异常标志的显式检查方式。
// js_try_begin() 保存当前深度并返回 0（正常进入）
// js_throw() 设置异常值和标志
// 每个 try 块在执行完毕后通过 js_check_exception() 检查是否有异常
// 如果有异常则跳转到 catch 块
//
// 由于 Cranelift 生成的是线性代码，try-catch 的控制流
// 已经在编译时由 codegen 处理（生成 try_block/catch_block/finally_block），
// 运行时只需提供异常值的存储和标志即可。

#define TRY_NEST_MAX 32

static int g_try_depth = 0;
static double g_exception_value;
static int g_has_exception = 0;

// js_try_begin: 进入 try 块，返回 0 表示正常进入
// 如果已有未处理的异常（从内层传播出来），返回 1 表示应进入 catch
int32_t js_try_begin(void) {
    if (g_try_depth >= TRY_NEST_MAX) {
        return 0;
    }
    g_try_depth++;
    // 如果有未处理的异常，直接进入 catch
    if (g_has_exception) {
        return 1;
    }
    return 0;
}

// js_try_end: 离开 try/catch 块
void js_try_end(void) {
    if (g_try_depth > 0) {
        g_try_depth--;
    }
}

// js_get_exception: 获取当前异常值
double js_get_exception(void) {
    return g_exception_value;
}

// js_clear_exception: 清除异常状态
void js_clear_exception(void) {
    uint64_t undef = UNDEFINED;
    g_has_exception = 0;
    memcpy(&g_exception_value, &undef, sizeof(double));
}

// js_has_exception_fn: 检查是否有未处理的异常
double js_has_exception_fn(void) {
    return g_has_exception ? bits_to_val(TRUE_VAL) : bits_to_val(FALSE_VAL);
}

// js_throw: 设置异常值
// 注意：实际的控制流跳转由 Cranelift 生成的代码处理
// js_throw 只设置异常值和标志，然后返回
// 调用者（codegen 生成的代码）负责检查异常并跳转到 catch 块
void js_throw(double value) {
    g_exception_value = value;
    g_has_exception = 1;
}

double js_math_random(void) {
    LARGE_INTEGER counter;
    QueryPerformanceCounter(&counter);
    uint64_t x = (uint64_t)counter.QuadPart;
    x ^= x >> 33;
    x *= 0xff51afd7ed558ccdULL;
    x ^= x >> 33;
    x *= 0xc4ceb9fe1a85ec53ULL;
    x ^= x >> 33;
    return (double)(x & 0x1FFFFFFFFFFFFFULL) / (double)0x20000000000000ULL;
}

double js_date_now(void) {
    FILETIME ft;
    GetSystemTimeAsFileTime(&ft);
    ULARGE_INTEGER uli;
    uli.LowPart = ft.dwLowDateTime;
    uli.HighPart = ft.dwHighDateTime;
    return (double)(uli.QuadPart / 10000ULL - 11644473600000ULL);
}

double js_math_floor(double val) {
    if (val >= 0) return (double)(int64_t)val;
    double f = (double)(int64_t)val;
    return (f > val) ? f - 1.0 : f;
}

double js_math_ceil(double val) {
    double f = (double)(int64_t)val;
    return (f < val) ? f + 1.0 : f;
}

double js_math_round(double val) {
    return (double)(int64_t)(val + 0.5);
}

double js_number_to_fixed(double val, double digits_d) {
    int digits = (int)digits_d;
    if (digits < 0) digits = 0;
    if (digits > 20) digits = 20;
    
    double scale = 1.0;
    for (int i = 0; i < digits; i++) scale *= 10.0;
    double scaled = val * scale;
    double rounded = js_math_floor(scaled);
    if (scaled - rounded >= 0.5) rounded += 1.0;
    if (rounded < 0 && scaled - rounded <= -0.5) rounded -= 1.0;
    
    double int_part_d = rounded / scale;
    int int_part = (int)int_part_d;
    
    char buf[64];
    int len = int_to_str(int_part, buf);
    
    if (digits > 0) {
        buf[len++] = '.';
        int frac_int = (int)(rounded < 0 ? -rounded : rounded);
        char frac_buf[32];
        int fl = int_to_str(frac_int, frac_buf);
        int pad = digits - fl;
        for (int i = 0; i < pad; i++) buf[len++] = '0';
        for (int i = 0; i < fl; i++) buf[len++] = frac_buf[i];
    }
    
    return js_string_new(buf, (uint32_t)len);
}

double js_metric_cpu_toFixed(double val) {
    return js_number_to_fixed(val, 2.0);
}

double js_metric_memory_toFixed(double val) {
    return js_number_to_fixed(val, 2.0);
}

double js_toFixed(double val, double digits) {
    return js_number_to_fixed(val, digits);
}

double js_to_fixed(double val, double digits) {
    return js_number_to_fixed(val, digits);
}

double js_json_stringify(double val) {
    uint64_t bits = val_to_bits(val);
    uint64_t tag = bits & TAG_MASK;
    
    if (bits == UNDEFINED) return js_string_from_static("undefined");
    if (bits == NULL_VAL) return js_string_from_static("null");
    if (bits == TRUE_VAL) return js_string_from_static("true");
    if (bits == FALSE_VAL) return js_string_from_static("false");
    if (tag == STRING_TAG) return val;
    if (tag == ARRAY_TAG || tag == OBJECT_TAG) return js_string_from_static("[object]");
    return js_number_to_string(val);
}

double js_json_parse(double val) {
    return val;
}

static int g_argc = 0;
static char** g_argv = NULL;

static void init_args(void) {
    if (g_argv) return;
    LPWSTR cmd_line = GetCommandLineW();
    int w_argc;
    LPWSTR* w_argv = CommandLineToArgvW(cmd_line, &w_argc);
    if (!w_argv) return;
    g_argc = w_argc;
    g_argv = (char**)HeapAlloc(GetProcessHeap(), 0, w_argc * sizeof(char*));
    if (!g_argv) { LocalFree(w_argv); return; }
    for (int i = 0; i < w_argc; i++) {
        int len = WideCharToMultiByte(CP_UTF8, 0, w_argv[i], -1, NULL, 0, NULL, NULL);
        g_argv[i] = (char*)HeapAlloc(GetProcessHeap(), 0, len);
        if (g_argv[i]) {
            WideCharToMultiByte(CP_UTF8, 0, w_argv[i], -1, g_argv[i], len, NULL, NULL);
        }
    }
    LocalFree(w_argv);
}

double argc(void) {
    init_args();
    return (double)g_argc;
}

double argv(double idx_val) {
    init_args();
    int idx = (int)idx_val;
    if (idx < 0 || idx >= g_argc || !g_argv || !g_argv[idx]) return bits_to_val(UNDEFINED);
    return js_string_from_static(g_argv[idx]);
}

double now_ms(void) {
    return js_date_now();
}

double sleep(double ms_val) {
    Sleep((DWORD)ms_val);
    return bits_to_val(UNDEFINED);
}

double http_get(double url_val) {
    return js_string_from_static("{}");
}

double file_append(double path_val, double content_val) {
    return bits_to_val(UNDEFINED);
}

double file_exists(double path_val) {
    return bits_to_val(FALSE_VAL);
}

double file_read(double path_val) {
    return js_string_from_static("");
}

double file_write(double path_val, double content_val) {
    return bits_to_val(UNDEFINED);
}

double format_timestamp(double ms_val) {
    int64_t ms = (int64_t)ms_val;
    int64_t total_sec = ms / 1000;
    int days_since_epoch = (int)(total_sec / 86400);
    int time_sec = (int)(total_sec % 86400);
    if (time_sec < 0) { time_sec += 86400; days_since_epoch--; }
    
    int hours = time_sec / 3600;
    int minutes = (time_sec % 3600) / 60;
    int seconds = time_sec % 60;
    
    int y = 1970;
    int days = days_since_epoch;
    while (1) {
        int dy = (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) ? 366 : 365;
        if (days < dy) break;
        days -= dy;
        y++;
    }
    int m = 1;
    int mdays[] = {31,28,31,30,31,30,31,31,30,31,30,31};
    if (y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) mdays[1] = 29;
    while (m <= 12 && days >= mdays[m-1]) { days -= mdays[m-1]; m++; }
    int d = days + 1;
    
    char buf[32];
    int len = 0;
    len = int_to_str(y, buf); buf[len++] = '-';
    if (m < 10) buf[len++] = '0'; { char tb[8]; int tl = int_to_str(m, tb); for (int i=0;i<tl;i++) buf[len++]=tb[i]; }
    buf[len++] = '-';
    if (d < 10) buf[len++] = '0'; { char tb[8]; int tl = int_to_str(d, tb); for (int i=0;i<tl;i++) buf[len++]=tb[i]; }
    buf[len++] = ' ';
    if (hours < 10) buf[len++] = '0'; { char tb[8]; int tl = int_to_str(hours, tb); for (int i=0;i<tl;i++) buf[len++]=tb[i]; }
    buf[len++] = ':';
    if (minutes < 10) buf[len++] = '0'; { char tb[8]; int tl = int_to_str(minutes, tb); for (int i=0;i<tl;i++) buf[len++]=tb[i]; }
    buf[len++] = ':';
    if (seconds < 10) buf[len++] = '0'; { char tb[8]; int tl = int_to_str(seconds, tb); for (int i=0;i<tl;i++) buf[len++]=tb[i]; }
    
    return js_string_new(buf, (uint32_t)len);
}

double http_listen(double port_val) {
    return bits_to_val(UNDEFINED);
}

double http_stop() {
    return bits_to_val(UNDEFINED);
}

double route_get(double path_val, double handler_val) {
    return bits_to_val(UNDEFINED);
}

double route_post(double path_val, double handler_val) {
    return bits_to_val(UNDEFINED);
}

double response_json(double val) {
    return val;
}

double response_text(double val) {
    return val;
}

double js_array_spread(double arr_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return arr_val;
    JsArray* src = (JsArray*)(arr_bits & PTR_MASK);
    if (!src) return bits_to_val(UNDEFINED);
    JsArray* dst = (JsArray*)js_malloc(sizeof(JsArray) + src->len * 8);
    if (!dst) return bits_to_val(UNDEFINED);
    dst->len = src->len;
    dst->capacity = src->len;
    for (uint32_t i = 0; i < src->len; i++) dst->data[i] = src->data[i];
    return bits_to_val(ARRAY_TAG | (uint64_t)dst);
}

double js_array_concat(double a_val, double b_val) {
    uint64_t a_bits = val_to_bits(a_val);
    uint64_t b_bits = val_to_bits(b_val);
    uint32_t a_len = 0, b_len = 0;
    uint64_t* a_data = NULL;
    uint64_t* b_data = NULL;
    if ((a_bits & TAG_MASK) == ARRAY_TAG) {
        JsArray* a = (JsArray*)(a_bits & PTR_MASK);
        if (a) { a_len = a->len; a_data = a->data; }
    }
    if ((b_bits & TAG_MASK) == ARRAY_TAG) {
        JsArray* b = (JsArray*)(b_bits & PTR_MASK);
        if (b) { b_len = b->len; b_data = b->data; }
    }
    uint32_t total = a_len + b_len;
    if (total == 0) total = 1;
    JsArray* result = (JsArray*)js_malloc(sizeof(JsArray) + total * 8);
    if (!result) return bits_to_val(UNDEFINED);
    result->len = 0;
    result->capacity = total;
    for (uint32_t i = 0; i < a_len; i++) result->data[result->len++] = a_data[i];
    for (uint32_t i = 0; i < b_len; i++) result->data[result->len++] = b_data[i];
    return bits_to_val(ARRAY_TAG | (uint64_t)result);
}

double js_array_slice(double arr_val, double start_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(UNDEFINED);
    JsArray* src = (JsArray*)(arr_bits & PTR_MASK);
    if (!src) return bits_to_val(UNDEFINED);
    uint32_t start = (uint32_t)start_val;
    if (start > src->len) start = src->len;
    uint32_t len = src->len - start;
    JsArray* dst = (JsArray*)js_malloc(sizeof(JsArray) + (len > 0 ? len : 1) * 8);
    if (!dst) return bits_to_val(UNDEFINED);
    dst->len = len;
    dst->capacity = len > 0 ? len : 1;
    for (uint32_t i = 0; i < len; i++) dst->data[i] = src->data[start + i];
    return bits_to_val(ARRAY_TAG | (uint64_t)dst);
}

double js_array_from_args(double start_idx_val) {
    return bits_to_val(UNDEFINED);
}

double js_function_apply(double fn_val, double args_val) {
    return bits_to_val(UNDEFINED);
}

double js_object_keys(double obj_val) {
    uint64_t obj_bits = val_to_bits(obj_val);
    if ((obj_bits & TAG_MASK) != OBJECT_TAG) {
        JsArray* arr = (JsArray*)js_malloc(sizeof(JsArray) + 8);
        if (!arr) return bits_to_val(UNDEFINED);
        arr->len = 0;
        arr->capacity = 1;
        return bits_to_val(ARRAY_TAG | (uint64_t)arr);
    }
    JsObject* obj = (JsObject*)(obj_bits & PTR_MASK);
    if (!obj) {
        JsArray* arr = (JsArray*)js_malloc(sizeof(JsArray) + 8);
        if (!arr) return bits_to_val(UNDEFINED);
        arr->len = 0;
        arr->capacity = 1;
        return bits_to_val(ARRAY_TAG | (uint64_t)arr);
    }
    JsArray* result = (JsArray*)js_malloc(sizeof(JsArray) + obj->size * 8);
    if (!result) return bits_to_val(UNDEFINED);
    result->len = obj->size;
    result->capacity = obj->size;
    for (uint32_t i = 0; i < obj->size; i++) {
        result->data[i] = obj->entries[i].key;
    }
    return bits_to_val(ARRAY_TAG | (uint64_t)result);
}

double js_object_spread(double obj_val) {
    return obj_val;
}

double js_object_set_computed(double placeholder, double key_val, double value_val) {
    return value_val;
}

double js_property_length(double val) {
    uint64_t bits = val_to_bits(val);
    uint64_t tag = bits & TAG_MASK;
    if (tag == STRING_TAG) {
        JsString* s = (JsString*)(bits & PTR_MASK);
        if (!s) return 0.0;
        return (double)s->len;
    }
    if (tag == ARRAY_TAG) {
        JsArray* a = (JsArray*)(bits & PTR_MASK);
        if (!a) return 0.0;
        return (double)a->len;
    }
    return bits_to_val(UNDEFINED);
}

static void js_free(void* ptr) {
    HeapFree(GetProcessHeap(), 0, ptr);
}

double js_push(double arr_val, double value) {
    return js_array_push(arr_val, value);
}

double js_pop(double arr_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(UNDEFINED);
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr || arr->len == 0) return bits_to_val(UNDEFINED);
    arr->len--;
    return bits_to_val(arr->data[arr->len]);
}

double js_shift(double arr_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(UNDEFINED);
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr || arr->len == 0) return bits_to_val(UNDEFINED);
    uint64_t first = arr->data[0];
    for (uint32_t i = 1; i < arr->len; i++) arr->data[i-1] = arr->data[i];
    arr->len--;
    return bits_to_val(first);
}

double js_unshift(double arr_val, double value) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return arr_val;
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr) return arr_val;
    if (arr->len >= arr->capacity) {
        uint32_t new_cap = arr->capacity * 2;
        JsArray* new_arr = (JsArray*)js_realloc(arr, sizeof(JsArray) + new_cap * 8);
        if (!new_arr) return arr_val;
        new_arr->capacity = new_cap;
        arr = new_arr;
    }
    for (uint32_t i = arr->len; i > 0; i--) arr->data[i] = arr->data[i-1];
    arr->data[0] = val_to_bits(value);
    arr->len++;
    return bits_to_val(ARRAY_TAG | (uint64_t)arr);
}

double js_indexOf(double arr_val, double search_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return -1.0;
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr) return -1.0;
    uint64_t search_bits = val_to_bits(search_val);
    for (uint32_t i = 0; i < arr->len; i++) {
        if (arr->data[i] == search_bits) return (double)i;
    }
    return -1.0;
}

double js_includes(double arr_val, double search_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(FALSE_VAL);
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr) return bits_to_val(FALSE_VAL);
    uint64_t search_bits = val_to_bits(search_val);
    for (uint32_t i = 0; i < arr->len; i++) {
        if (arr->data[i] == search_bits) return bits_to_val(TRUE_VAL);
    }
    return bits_to_val(FALSE_VAL);
}

double js_join(double arr_val, double sep_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(UNDEFINED);
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr) return bits_to_val(UNDEFINED);
    uint64_t sep_bits = val_to_bits(sep_val);
    const char* sep_data = ",";
    uint32_t sep_len = 1;
    JsString* sep_js = NULL;
    if ((sep_bits & TAG_MASK) == STRING_TAG) {
        sep_js = (JsString*)(sep_bits & PTR_MASK);
        if (sep_js) { sep_data = (const char*)sep_js->data; sep_len = sep_js->len; }
    }
    uint32_t total_len = 0;
    if (arr->len > 0) total_len += (arr->len - 1) * sep_len;
    char* parts[256];
    uint32_t parts_len[256];
    uint32_t n = arr->len > 256 ? 256 : arr->len;
    for (uint32_t i = 0; i < n; i++) {
        char buf[64];
        int len = 0;
        uint64_t elem = arr->data[i];
        if (elem == UNDEFINED) { len = 9; parts[i] = (char*)js_malloc(len); memcpy(parts[i], "undefined", len); }
        else if (elem == NULL_VAL) { len = 4; parts[i] = (char*)js_malloc(len); memcpy(parts[i], "null", len); }
        else if (elem == TRUE_VAL) { len = 4; parts[i] = (char*)js_malloc(len); memcpy(parts[i], "true", len); }
        else if (elem == FALSE_VAL) { len = 5; parts[i] = (char*)js_malloc(len); memcpy(parts[i], "false", len); }
        else if ((elem & TAG_MASK) == STRING_TAG) {
            JsString* es = (JsString*)(elem & PTR_MASK);
            if (es) { len = es->len; parts[i] = (char*)js_malloc(len > 0 ? len : 1); memcpy(parts[i], es->data, len); }
            else { len = 4; parts[i] = (char*)js_malloc(len); memcpy(parts[i], "null", len); }
        } else {
            double v = bits_to_val(elem);
            int64_t num = (int64_t)v;
            len = int64_to_str(num, buf);
            parts[i] = (char*)js_malloc(len > 0 ? len : 1);
            memcpy(parts[i], buf, len);
        }
        parts_len[i] = (uint32_t)len;
        total_len += (uint32_t)len;
    }
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + (total_len > 0 ? total_len : 1));
    if (!result) return bits_to_val(UNDEFINED);
    result->len = 0;
    result->hash = 0;
    for (uint32_t i = 0; i < n; i++) {
        if (i > 0) {
            if (sep_js) { memcpy(result->data + result->len, sep_data, sep_len); result->len += sep_len; }
            else { result->data[result->len++] = ','; }
        }
        memcpy(result->data + result->len, parts[i], parts_len[i]);
        result->len += parts_len[i];
        js_free(parts[i]);
    }
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_split(double str_val, double sep_val) {
    return js_string_split(str_val, sep_val);
}

double js_replace(double str_val, double search_val, double replace_val) {
    return js_string_replace(str_val, search_val, replace_val);
}

double js_trim(double str_val) {
    uint64_t str_bits = val_to_bits(str_val);
    if ((str_bits & TAG_MASK) != STRING_TAG) return bits_to_val(UNDEFINED);
    JsString* s = (JsString*)(str_bits & PTR_MASK);
    if (!s) return bits_to_val(UNDEFINED);
    uint32_t start = 0, end = s->len;
    while (start < end && (s->data[start] == ' ' || s->data[start] == '\t' || s->data[start] == '\n' || s->data[start] == '\r')) start++;
    while (end > start && (s->data[end-1] == ' ' || s->data[end-1] == '\t' || s->data[end-1] == '\n' || s->data[end-1] == '\r')) end--;
    uint32_t len = end - start;
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + (len > 0 ? len : 1));
    if (!result) return bits_to_val(UNDEFINED);
    result->len = len;
    result->hash = 0;
    for (uint32_t i = 0; i < len; i++) result->data[i] = s->data[start + i];
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_toUpperCase(double str_val) {
    uint64_t str_bits = val_to_bits(str_val);
    if ((str_bits & TAG_MASK) != STRING_TAG) return bits_to_val(UNDEFINED);
    JsString* s = (JsString*)(str_bits & PTR_MASK);
    if (!s) return bits_to_val(UNDEFINED);
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + s->len);
    if (!result) return bits_to_val(UNDEFINED);
    result->len = s->len;
    result->hash = 0;
    for (uint32_t i = 0; i < s->len; i++) {
        char c = s->data[i];
        if (c >= 'a' && c <= 'z') c -= 32;
        result->data[i] = c;
    }
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_toLowerCase(double str_val) {
    uint64_t str_bits = val_to_bits(str_val);
    if ((str_bits & TAG_MASK) != STRING_TAG) return bits_to_val(UNDEFINED);
    JsString* s = (JsString*)(str_bits & PTR_MASK);
    if (!s) return bits_to_val(UNDEFINED);
    JsString* result = (JsString*)js_malloc(sizeof(JsString) + s->len);
    if (!result) return bits_to_val(UNDEFINED);
    result->len = s->len;
    result->hash = 0;
    for (uint32_t i = 0; i < s->len; i++) {
        char c = s->data[i];
        if (c >= 'A' && c <= 'Z') c += 32;
        result->data[i] = c;
    }
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_startsWith(double str_val, double search_val) {
    uint64_t str_bits = val_to_bits(str_val);
    uint64_t search_bits = val_to_bits(search_val);
    if ((str_bits & TAG_MASK) != STRING_TAG || (search_bits & TAG_MASK) != STRING_TAG) return bits_to_val(FALSE_VAL);
    JsString* s = (JsString*)(str_bits & PTR_MASK);
    JsString* q = (JsString*)(search_bits & PTR_MASK);
    if (!s || !q) return bits_to_val(FALSE_VAL);
    if (q->len > s->len) return bits_to_val(FALSE_VAL);
    for (uint32_t i = 0; i < q->len; i++) {
        if (s->data[i] != q->data[i]) return bits_to_val(FALSE_VAL);
    }
    return bits_to_val(TRUE_VAL);
}

double js_endsWith(double str_val, double search_val) {
    uint64_t str_bits = val_to_bits(str_val);
    uint64_t search_bits = val_to_bits(search_val);
    if ((str_bits & TAG_MASK) != STRING_TAG || (search_bits & TAG_MASK) != STRING_TAG) return bits_to_val(FALSE_VAL);
    JsString* s = (JsString*)(str_bits & PTR_MASK);
    JsString* q = (JsString*)(search_bits & PTR_MASK);
    if (!s || !q) return bits_to_val(FALSE_VAL);
    if (q->len > s->len) return bits_to_val(FALSE_VAL);
    for (uint32_t i = 0; i < q->len; i++) {
        if (s->data[s->len - q->len + i] != q->data[i]) return bits_to_val(FALSE_VAL);
    }
    return bits_to_val(TRUE_VAL);
}

double js_object_values(double obj_val) {
    uint64_t obj_bits = val_to_bits(obj_val);
    if ((obj_bits & TAG_MASK) != OBJECT_TAG) return bits_to_val(UNDEFINED);
    JsObject* obj = (JsObject*)(obj_bits & PTR_MASK);
    if (!obj) return bits_to_val(UNDEFINED);
    JsArray* result = (JsArray*)js_malloc(sizeof(JsArray) + obj->size * 8);
    if (!result) return bits_to_val(UNDEFINED);
    result->len = obj->size;
    result->capacity = obj->size;
    for (uint32_t i = 0; i < obj->size; i++) {
        result->data[i] = obj->entries[i].value;
    }
    return bits_to_val(ARRAY_TAG | (uint64_t)result);
}

double js_object_assign(double target_val, double source_val) {
    uint64_t target_bits = val_to_bits(target_val);
    uint64_t source_bits = val_to_bits(source_val);
    if ((target_bits & TAG_MASK) != OBJECT_TAG) return target_val;
    if ((source_bits & TAG_MASK) != OBJECT_TAG) return target_val;
    JsObject* target = (JsObject*)(target_bits & PTR_MASK);
    JsObject* source = (JsObject*)(source_bits & PTR_MASK);
    if (!target || !source) return target_val;
    for (uint32_t i = 0; i < source->size; i++) {
        double key_val = bits_to_val(source->entries[i].key);
        double value_val = bits_to_val(source->entries[i].value);
        js_object_set(target_val, key_val, value_val);
    }
    return target_val;
}

double js_array_is_array(double val) {
    uint64_t bits = val_to_bits(val);
    if ((bits & TAG_MASK) == ARRAY_TAG) return bits_to_val(TRUE_VAL);
    return bits_to_val(FALSE_VAL);
}

double js_in(double key_val, double obj_val) {
    uint64_t obj_bits = val_to_bits(obj_val);
    if ((obj_bits & TAG_MASK) == OBJECT_TAG) {
        JsObject* obj = (JsObject*)(obj_bits & PTR_MASK);
        if (obj) {
            uint64_t key_bits = val_to_bits(key_val);
            for (uint32_t i = 0; i < obj->size; i++) {
                if (obj->entries[i].key == key_bits) return bits_to_val(TRUE_VAL);
            }
        }
    }
    if ((obj_bits & TAG_MASK) == ARRAY_TAG) {
        JsArray* arr = (JsArray*)(obj_bits & PTR_MASK);
        if (arr) {
            int idx = (int)key_val;
            if (idx >= 0 && (uint32_t)idx < arr->len) return bits_to_val(TRUE_VAL);
        }
    }
    return bits_to_val(FALSE_VAL);
}
