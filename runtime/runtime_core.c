#include "runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Callback dispatcher for async events (MQTT, timers, etc.)
static void (*tsn_callback_dispatcher)(uint64_t, uint64_t, uint64_t) = NULL;

void js_set_callback_dispatcher(uint64_t dispatcher) {
    tsn_callback_dispatcher = (void (*)(uint64_t, uint64_t, uint64_t))dispatcher;
}

void tsn_dispatch_callback(uint64_t callback_id, uint64_t arg1, uint64_t arg2) {
    if (tsn_callback_dispatcher) {
        tsn_callback_dispatcher(callback_id, arg1, arg2);
    }
}