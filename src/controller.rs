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

        for (i, flag) in flags.iter_mut().enumerate() {
            flag.controller_present = nth_bit(value, i);
            flag.has_mempak = nth_bit(value, i + 4);
            flag.has_rumblepak = nth_bit(value, i + 8);
        }

        flags
    }

    pub fn to_u32(controllers: &[Flags; 4]) -> u32 {
        let mut value = 0;

        for (i, flag) in controllers.iter().enumerate() {
            value |= (flag.controller_present as u32) << i;
            value |= (flag.has_mempak as u32) << (i + 4);
            value |= (flag.has_rumblepak as u32) << (i + 8);
        }

        value
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

impl From<u32> for Input {
    // big endian
    fn from(mut value: u32) -> Self {
        let right_dpad = value & 0x01 != 0;
        value = value.shr(1);
        let left_dpad = value & 0x01 != 0;
        value = value.shr(1);
        let down_dpad = value & 0x01 != 0;
        value = value.shr(1);
        let up_dpad = value & 0x01 != 0;
        value = value.shr(1);
        let start = value & 0x01 != 0;
        value = value.shr(1);
        let z_button = value & 0x01 != 0;
        value = value.shr(1);
        let b_button = value & 0x01 != 0;
        value = value.shr(1);
        let a_button = value & 0x01 != 0;
        value = value.shr(1);
        let right_cbutton = value & 0x01 != 0;
        value = value.shr(1);
        let left_cbutton = value & 0x01 != 0;
        value = value.shr(1);
        let down_cbutton = value & 0x01 != 0;
        value = value.shr(1);
        let up_cbutton = value & 0x01 != 0;
        value = value.shr(1);
        let right_shoulder = value & 0x01 != 0;
        value = value.shr(1);
        let left_shoulder = value & 0x01 != 0;
        value = value.shr(1);
        let reserved_1 = value & 0x01 != 0;
        value = value.shr(1);
        let reserved_2 = value & 0x01 != 0;
        value = value.shr(1);
        let x_axis = (value & 0xFF) as i8;
        value = value.shr(8);
        let y_axis = value as i8;

        Input {
            up_dpad,
            down_dpad,
            left_dpad,
            right_dpad,
            start,
            z_button,
            a_button,
            b_button,
            right_shoulder,
            left_shoulder,
            up_cbutton,
            down_cbutton,
            left_cbutton,
            right_cbutton,
            reserved_1,
            reserved_2,
            x_axis,
            y_axis,
        }
    }
}
