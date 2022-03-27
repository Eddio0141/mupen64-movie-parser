use std::ops::Shr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flags {
    pub controller_present: bool,
    pub has_mempak: bool,
    pub has_rumblepak: bool,
}

impl Flags {
    pub fn from_u32(value: u32) -> [Flags; 4] {
        let mut flags = [Flags {
            controller_present: false,
            has_mempak: false,
            has_rumblepak: false,
        }; 4];

        for i in 0..4 {
            flags[i].controller_present = nth_bit(value, i);
            flags[i].has_mempak = nth_bit(value, i + 4);
            flags[i].has_rumblepak = nth_bit(value, i + 8);
        }

        flags
    }
}

fn nth_bit(value: u32, n: usize) -> bool {
    value.shr(n) & 0x01 != 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Input {
    pub up_dpad: bool,
    pub down_dpad: bool,
    pub left_dpad: bool,
    pub right_dpad: bool,
    pub start: bool,
    pub z_button: bool,
    pub a_button: bool,
    pub b_button: bool,
    pub right_shoulder: bool,
    pub left_shoulder: bool,
    pub up_cbutton: bool,
    pub down_cbutton: bool,
    pub left_cbutton: bool,
    pub right_cbutton: bool,
    pub reserved_1: bool,
    pub reserved_2: bool,

    pub x_axis: i8,
    pub y_axis: i8,
}
