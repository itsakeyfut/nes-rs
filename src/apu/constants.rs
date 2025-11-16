//! APU constants and lookup tables

/// Length counter lookup table
/// Maps the 5-bit length counter load value to the actual counter value
pub const LENGTH_COUNTER_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

/// Duty cycle patterns for pulse channels
/// Each pattern is 8 steps, representing one full cycle of the square wave
pub const DUTY_PATTERNS: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5% duty cycle
    [0, 1, 1, 0, 0, 0, 0, 0], // 25% duty cycle
    [0, 1, 1, 1, 1, 0, 0, 0], // 50% duty cycle
    [1, 0, 0, 1, 1, 1, 1, 1], // 75% duty cycle (inverted 25%)
];

/// Triangle wave sequence for triangle channel
/// 32-step sequence from 15 down to 0, then back up to 15
pub const TRIANGLE_SEQUENCE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

/// Noise channel period lookup table
/// Maps the 4-bit period value to the actual timer period (in CPU cycles)
/// NTSC values
pub const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];
