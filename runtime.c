#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
    memcpy(&result, &bits, 8);
    return result;
}

static uint64_t val_to_bits(double val) {
    uint64_t bits;
    memcpy(&bits, &val, 8);
    return bits;
}

double js_print(double val) {
    uint64_t bits = val_to_bits(val);
    uint64_t tag = bits & TAG_MASK;

    if (bits == UNDEFINED) printf("undefined");
    else if (bits == NULL_VAL) printf("null");
    else if (bits == TRUE_VAL) printf("true");
    else if (bits == FALSE_VAL) printf("false");
    else if (tag == STRING_TAG) {
        JsString* s = (JsString*)(bits & PTR_MASK);
        if (s) printf("%.*s", s->len, s->data);
    }
    else if (tag == ARRAY_TAG) {
        JsArray* a = (JsArray*)(bits & PTR_MASK);
        if (a) printf("[Array:%u]", a->len);
    }
    else if (tag == OBJECT_TAG) {
        JsObject* o = (JsObject*)(bits & PTR_MASK);
        if (o) printf("[Object:%u]", o->size);
    }
    else printf("%g", val);
    printf("\n");
    return val;
}

double js_array_new(uint32_t capacity) {
    if (capacity == 0) capacity = 8;
    JsArray* arr = (JsArray*)malloc(sizeof(JsArray) + capacity * 8);
    if (!arr) return bits_to_val(UNDEFINED);
    arr->len = 0;
    arr->capacity = capacity;
    return bits_to_val(ARRAY_TAG | (uint64_t)arr);
}

double js_array_get(double arr_val, double idx_val) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return bits_to_val(UNDEFINED);
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    uint32_t idx = (uint32_t)val_to_bits(idx_val);
    if (!arr || idx >= arr->len) return bits_to_val(UNDEFINED);
    return bits_to_val(arr->data[idx]);
}

double js_typeof(double val) {
    uint64_t bits = val_to_bits(val);
    const char* t = "number";
    if (bits == UNDEFINED) t = "undefined";
    else if (bits == NULL_VAL) t = "object";
    else if (bits == TRUE_VAL || bits == FALSE_VAL) t = "boolean";
    else if ((bits & TAG_MASK) == STRING_TAG) t = "string";
    else if ((bits & TAG_MASK) == ARRAY_TAG || (bits & TAG_MASK) == OBJECT_TAG) t = "object";
    uint32_t len = (uint32_t)strlen(t);
    JsString* s = (JsString*)malloc(sizeof(JsString) + len);
    s->len = len;
    s->hash = 0;
    memcpy(s->data, t, len);
    return bits_to_val(STRING_TAG | (uint64_t)s);
}

double js_array_push(double arr_val, double value) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return arr_val;
    
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    if (!arr) return arr_val;
    
    if (arr->len >= arr->capacity) {
        uint32_t new_cap = arr->capacity * 2;
        JsArray* new_arr = (JsArray*)realloc(arr, sizeof(JsArray) + new_cap * 8);
        if (!new_arr) return arr_val;
        new_arr->capacity = new_cap;
        arr = new_arr;
    }
    
    arr->data[arr->len++] = val_to_bits(value);
    return arr_val;
}

double js_array_set(double arr_val, double idx_val, double value) {
    uint64_t arr_bits = val_to_bits(arr_val);
    if ((arr_bits & TAG_MASK) != ARRAY_TAG) return value;
    
    JsArray* arr = (JsArray*)(arr_bits & PTR_MASK);
    uint32_t idx = (uint32_t)val_to_bits(idx_val);
    
    if (!arr) return value;
    
    if (idx >= arr->len) {
        while (arr->len <= idx) {
            if (arr->len >= arr->capacity) {
                uint32_t new_cap = arr->capacity * 2;
                JsArray* new_arr = (JsArray*)realloc(arr, sizeof(JsArray) + new_cap * 8);
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
    JsObject* obj = (JsObject*)malloc(sizeof(JsObject) + 8 * sizeof(ObjectEntry));
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
        JsObject* new_obj = (JsObject*)realloc(obj, sizeof(JsObject) + new_cap * sizeof(ObjectEntry));
        if (!new_obj) return value_val;
        new_obj->capacity = new_cap;
        obj = new_obj;
    }
    
    obj->entries[obj->size].key = key_bits;
    obj->entries[obj->size].value = value_bits;
    obj->size++;
    
    return value_val;
}

double js_string_new(const char* s, uint32_t len) {
    JsString* str = (JsString*)malloc(sizeof(JsString) + len);
    if (!str) return bits_to_val(UNDEFINED);
    
    str->len = len;
    str->hash = 0;
    memcpy(str->data, s, len);
    
    for (uint32_t i = 0; i < len; i++) {
        str->hash = str->hash * 31 + s[i];
    }
    
    return bits_to_val(STRING_TAG | (uint64_t)str);
}

double js_string_concat(double a_val, double b_val) {
    uint64_t a_bits = val_to_bits(a_val);
    uint64_t b_bits = val_to_bits(b_val);
    
    if ((a_bits & TAG_MASK) != STRING_TAG || (b_bits & TAG_MASK) != STRING_TAG) {
        return bits_to_val(UNDEFINED);
    }
    
    JsString* a = (JsString*)(a_bits & PTR_MASK);
    JsString* b = (JsString*)(b_bits & PTR_MASK);
    
    if (!a || !b) return bits_to_val(UNDEFINED);
    
    uint32_t new_len = a->len + b->len;
    JsString* result = (JsString*)malloc(sizeof(JsString) + new_len);
    if (!result) return bits_to_val(UNDEFINED);
    
    result->len = new_len;
    memcpy(result->data, a->data, a->len);
    memcpy(result->data + a->len, b->data, b->len);
    
    result->hash = 0;
    for (uint32_t i = 0; i < new_len; i++) {
        result->hash = result->hash * 31 + result->data[i];
    }
    
    return bits_to_val(STRING_TAG | (uint64_t)result);
}

double js_add(double a, double b) { return a + b; }
double js_sub(double a, double b) { return a - b; }
double js_mul(double a, double b) { return a * b; }
double js_div(double a, double b) { return b != 0 ? a / b : bits_to_val(UNDEFINED); }
double js_mod(double a, double b) { return b != 0 ? (double)((int64_t)a % (int64_t)b) : bits_to_val(UNDEFINED); }

double js_eq(double a, double b) { return bits_to_val(a == b ? TRUE_VAL : FALSE_VAL); }
double js_ne(double a, double b) { return bits_to_val(a != b ? TRUE_VAL : FALSE_VAL); }
double js_lt(double a, double b) { return bits_to_val(a < b ? TRUE_VAL : FALSE_VAL); }
double js_le(double a, double b) { return bits_to_val(a <= b ? TRUE_VAL : FALSE_VAL); }
double js_gt(double a, double b) { return bits_to_val(a > b ? TRUE_VAL : FALSE_VAL); }
double js_ge(double a, double b) { return bits_to_val(a >= b ? TRUE_VAL : FALSE_VAL); }
