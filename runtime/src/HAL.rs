// TODO: hardware abstraction layer
// hardware interface for servos go here
// @Denys

// in C++
// typedef struct {
//     uint8_t torque_switch;         // 0x28 (1 byte)
//     uint8_t acceleration;          // 0x29 (1 byte)
//     int16_t target_location;       // 0x2A (2 bytes)
//     uint16_t running_time;         // 0x2C (2 bytes)
//     uint16_t running_speed;        // 0x2E (2 bytes)
//     uint16_t torque_limit;         // 0x30 (2 bytes)
//     uint8_t reserved1[6];          // 0x32-0x37 (6 bytes, reserved)
//     uint8_t lock_mark;             // 0x37 (1 byte)
//     int16_t current_location;      // 0x38 (2 bytes)
//     int16_t current_speed;         // 0x3A (2 bytes)
//     int16_t current_load;          // 0x3C (2 bytes)
//     uint8_t current_voltage;       // 0x3E (1 byte)
//     uint8_t current_temperature;   // 0x3F (1 byte)
//     uint8_t async_write_flag;      // 0x40 (1 byte)
//     uint8_t servo_status;          // 0x41 (1 byte)
//     uint8_t mobile_sign;           // 0x42 (1 byte)
//     uint8_t reserved2[2];          // 0x43-0x44 (2 bytes, reserved)
//     uint16_t current_current;      // 0x45 (2 bytes)
// } ServoInfo;

// Something like this?
pub struct ServoInfo {
    pub torque_switch: u8,
    pub acceleration: u8,
    pub target_location: i16,
    pub running_time: u16,
    pub running_speed: u16,
    pub torque_limit: u16,
    pub lock_mark: u8,
    pub current_location: i16,
    pub current_speed: i16,
    pub current_load: i16,
    pub current_voltage: u8,
    pub current_temperature: u8,
    pub async_write_flag: u8,
    pub servo_status: u8,
    pub mobile_sign: u8,
    pub current_current: u16,
}
