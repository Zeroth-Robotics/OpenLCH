#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
#include "driver/uart.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "freertos/semphr.h"
#include "esp_log.h"

#define UART_NUM 1
#define BUF_SIZE (1024)

static const char* TAG = "ServoController";

// Error codes
enum {
    SERVO_ERROR_OVER_TEMPERATURE = 1,
    SERVO_ERROR_OVER_VOLTAGE = 2,
    SERVO_ERROR_LOCKED_ROTOR = 4
};

// Command codes
enum {
    SERVO_MOVE_TIME_WRITE = 1,
    // Add other commands here
    SERVO_ID_READ = 14,
    SERVO_POS_READ = 28,
    // Add all other command codes
};

// Data structure equivalent to ServoController
typedef struct {
    uart_port_t uart_num;
    int timeout_ms;
    SemaphoreHandle_t lock;
} ServoController;

// Function to initialize the UART
void uart_init(ServoController *controller) {
    uart_config_t uart_config = {
        .baud_rate = 115200,
        .data_bits = UART_DATA_8_BITS,
        .parity    = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT,
    };

    ESP_ERROR_CHECK(uart_param_config(UART_NUM_2, &uart_config));
    ESP_ERROR_CHECK(uart_driver_install(UART_NUM_2, 1024, 0, 0, NULL, 0));
    ESP_ERROR_CHECK(uart_set_pin(UART_NUM_2, 9, 10, 11, 12));
}

// Function to calculate lower byte
uint8_t lower_byte(int value) {
    return value & 0xFF;
}

// Function to calculate higher byte
uint8_t higher_byte(int value) {
    return (value >> 8) & 0xFF;
}

// Function to clamp a value within a range
int clamp(int range_min, int range_max, int value) {
    if (value < range_min) return range_min;
    if (value > range_max) return range_max;
    return value;
}

// Function to calculate checksum
uint8_t calculate_checksum(uint8_t* packet, int length) {
    int sum = 0;
    for (int i = 2; i < length-1; ++i) {
        sum += packet[i];
    }
    return 255 - (sum % 256);
}

static uint16_t word(uint8_t low, uint8_t high) {
    return (uint16_t)low + ((uint16_t)high << 8);
}

// Function to send a command to the servo
void send_command(ServoController *controller, uint8_t servo_id, uint8_t command, uint8_t *params, int param_length) {
    int packet_length = 2 + 3 + param_length + 1;
    uint8_t *packet = malloc(packet_length);
    packet[0] = 0x55;
    packet[1] = 0x55;
    packet[2] = servo_id;
    packet[3] = 3 + param_length;
    packet[4] = command;
    memcpy(&packet[5], params, param_length);
    packet[packet_length - 1] = calculate_checksum(packet, packet_length);

    xSemaphoreTake(controller->lock, portMAX_DELAY);
    uart_write_bytes(controller->uart_num, (const char *)packet, packet_length);
    xSemaphoreGive(controller->lock);

    free(packet);
}

// Example function: Move Servo
void move_servo(ServoController *controller, uint8_t servo_id, int position, int time) {
    uint8_t params[4];
    position = clamp(0, 1000, position);
    time = clamp(0, 30000, time);

    params[0] = lower_byte(position);
    params[1] = higher_byte(position);
    params[2] = lower_byte(time);
    params[3] = higher_byte(time);

    send_command(controller, servo_id, SERVO_MOVE_TIME_WRITE, params, 4);
}

// Initialization function for ServoController
void init_servo_controller(ServoController *controller, uart_port_t uart_num, int timeout_ms) {
    controller->uart_num = uart_num;
    controller->timeout_ms = timeout_ms;
    controller->lock = xSemaphoreCreateMutex();
    uart_init(controller);
}

uint8_t sum_params(uint8_t *data, uint8_t length) {
    uint8_t sum = 0;
    for (uint8_t i = 5; i < length + 2; i++) {  // Adjusted to sum the parameters correctly
        sum += data[i];
    }
    return sum;
}

bool wait_for_response(ServoController *controller, uint8_t servo_id, uint8_t command, uint8_t *response, size_t response_size, int timeout_ms) {
    TickType_t start_tick = xTaskGetTickCount();
    size_t index = 0;

    while (1) {
        // Read one byte at a time
        size_t len = uart_read_bytes(controller->uart_num, &response[index], 1, pdMS_TO_TICKS(10));
        if (len > 0) {
            // ESP_LOGE(TAG, "RCV i %d, %d", index, response[index]);
            // Check for the double 0x55 header
            if (index == 0 && response[0] != 0x55) {
                continue;  // Keep waiting for the first 0x55
            }
            if (index == 1 && response[1] != 0x55) {
                index = 0;  // Reset if the second byte isn't 0x55
                continue;
            }
            index++;

            // After reading the header and some data, verify packet length
            if (index == 5) {
                uint8_t length = response[3];
                if (length > 7) {
                    ESP_LOGE(TAG, "Invalid packet length");
                    index = 0;  // Reset and continue waiting
                    continue;
                }
            }

            // After reading all expected bytes, validate checksum
            if (index >= response_size) {
                uint8_t sid = response[2];
                uint8_t length = response[3];
                uint8_t cmd = response[4];
                uint8_t checksum = 255 - ((sid + length + cmd + sum_params(response, length)) % 256);

                if (response[index - 1] == checksum || 1) {
                    if (cmd == command && (servo_id == SERVO_ID_READ || sid == servo_id)) {
                        return true;  // Valid response
                    } else {
                        ESP_LOGW(TAG, "Unexpected command or servo ID");
                        index = 0;  // Reset and continue waiting
                    }
                } else {
                    ESP_LOGE(TAG, "Invalid checksum");
                    index = 0;  // Reset and continue waiting
                }
            }
        }

        // Check for timeout
        if ((xTaskGetTickCount() - start_tick) * portTICK_PERIOD_MS >= timeout_ms) {
            return false; // Timeout
        }
    }
}


int get_position(ServoController *controller, uint8_t servo_id, int timeout_ms) {
    uint8_t command = SERVO_POS_READ;
    uint8_t response[7];  // Adjust the size based on expected response length
    int position;
    
    send_command(controller, servo_id, command, NULL, 0);
    
    if (wait_for_response(controller, servo_id, command, response, sizeof(response), timeout_ms)) {
        position = word(response[5], response[6]);
        if (position > 32767) {
            position -= 65536;
        }
        return position;
    } else {
        // Handle timeout or error - returning some error code or using logging
        ESP_LOGE(TAG, "Failed to get position from servo %d", servo_id);
        return -1; // Indicate failure
    }
}
