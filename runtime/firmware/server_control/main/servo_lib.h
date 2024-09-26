#ifndef SERVO_CONTROLLER_H
#define SERVO_CONTROLLER_H

#include <stdint.h>
#include "driver/uart.h"
#include "freertos/semphr.h"

// Error codes
#define SERVO_ERROR_OVER_TEMPERATURE 1
#define SERVO_ERROR_OVER_VOLTAGE 2
#define SERVO_ERROR_LOCKED_ROTOR 4

// Command codes
#define SERVO_MOVE_TIME_WRITE 1
#define SERVO_ID_READ 14
#define SERVO_POS_READ 28
// Add all other command codes

// ServoController data structure
typedef struct {
    uart_port_t uart_num;
    int timeout_ms;
    SemaphoreHandle_t lock;
} ServoController;

// Function prototypes
void uart_init(ServoController *controller);
uint8_t lower_byte(int value);
uint8_t higher_byte(int value);
int clamp(int range_min, int range_max, int value);
uint8_t calculate_checksum(uint8_t* packet, int length);
void send_command(ServoController *controller, uint8_t servo_id, uint8_t command, uint8_t *params, int param_length);
void move_servo(ServoController *controller, uint8_t servo_id, int position, int time);
void init_servo_controller(ServoController *controller, uart_port_t uart_num, int timeout_ms);
int get_position(ServoController *controller, uint8_t servo_id, int timeout_ms);
bool wait_for_response(ServoController *controller, uint8_t servo_id, uint8_t command, uint8_t *response, size_t response_size, int timeout_ms);

#endif // SERVO_CONTROLLER_H
