#include "runtime.h"
#include <stdio.h>

// DOM API (minimal implementation - no actual rendering)
uint64_t js_dom_create_element(const char *tag) {
    printf("[DOM] createElement: %s\n", tag);
    return bits_to_val(1);
}

uint64_t js_dom_get_element_by_id(const char *id) {
    printf("[DOM] getElementById: %s\n", id);
    return bits_to_val(1);
}

uint64_t js_dom_create_text_node(const char *text) {
    printf("[DOM] createTextNode: %s\n", text);
    return bits_to_val(2);
}

uint64_t js_dom_append_child(uint64_t parent, uint64_t child) {
    printf("[DOM] appendChild: %lu <- %lu\n", (unsigned long)parent, (unsigned long)child);
    return bits_to_val(parent);
}

void js_dom_set_attribute(uint64_t elem, const char *name, const char *value) {
    printf("[DOM] setAttribute: elem=%lu, %s=%s\n", (unsigned long)elem, name, value);
}

void js_dom_set_text_content(uint64_t elem, const char *text) {
    printf("[DOM] textContent: elem=%lu, %s\n", (unsigned long)elem, text);
}

void js_dom_set_value(uint64_t elem, const char *value) {
    printf("[DOM] value: elem=%lu, %s\n", (unsigned long)elem, value);
}

void js_dom_add_event_listener(uint64_t elem, uint64_t event_type, uint64_t callback_id) {
    printf("[DOM] addEventListener: elem=%lu, type=%lu, callback=%lu\n",
           (unsigned long)elem, (unsigned long)event_type, (unsigned long)callback_id);
}

void js_dom_main_loop() {
    printf("[DOM] mainLoop started (simulated, press Ctrl+C to exit)\n");
    while (1) {
        // Platform-specific event loop
    }
}