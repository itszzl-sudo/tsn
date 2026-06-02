#include "runtime.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <mosquitto.h>
#include <unistd.h>

static struct mosquitto *mqtt_client = NULL;
static int mqtt_connected = 0;

void on_mqtt_message(struct mosquitto *mosq, void *userdata, const struct mosquitto_message *message) {
    char payload[256];
    memcpy(payload, message->payload, message->payloadlen);
    payload[message->payloadlen] = '\0';
    
    printf("[MQTT] Received: %s = %s\n", message->topic, payload);
    
    uint64_t topic_val = bits_to_val(STRING_TAG | (uint64_t)message->topic);
    uint64_t payload_val = bits_to_val(STRING_TAG | (uint64_t)payload);
    tsn_dispatch_callback(0, topic_val, payload_val);
}

uint64_t js_mqtt_connect(const char *broker, const char *client_id) {
    printf("[MQTT] Connecting to %s as %s\n", broker, client_id);
    
    mosquitto_lib_init();
    mqtt_client = mosquitto_new(client_id, true, NULL);
    
    if (!mqtt_client) {
        fprintf(stderr, "Error: Out of memory.\n");
        return bits_to_val(0);
    }
    
    mosquitto_message_callback_set(mqtt_client, on_mqtt_message);
    
    if (mosquitto_connect(mqtt_client, broker, 1883, 60)) {
        fprintf(stderr, "Unable to connect.\n");
        return bits_to_val(0);
    }
    
    mqtt_connected = 1;
    return bits_to_val(1);
}

void js_mqtt_disconnect(uint64_t conn) {
    printf("[MQTT] Disconnecting...\n");
    if (mqtt_client) {
        mosquitto_disconnect(mqtt_client);
        mosquitto_destroy(mqtt_client);
        mqtt_client = NULL;
    }
    mqtt_connected = 0;
    mosquitto_lib_cleanup();
}

void js_mqtt_subscribe(uint64_t conn, const char *topic, uint64_t callback_id) {
    printf("[MQTT] Subscribe: %s (callback=%lu)\n", topic, (unsigned long)callback_id);
    if (mqtt_client) {
        mosquitto_subscribe(mqtt_client, NULL, topic, 0);
    }
}

void js_mqtt_publish(uint64_t conn, const char *topic, const char *payload) {
    printf("[MQTT] Publish: %s = %s\n", topic, payload);
    if (mqtt_client) {
        mosquitto_publish(mqtt_client, NULL, topic, strlen(payload), payload, 0, false);
    }
}