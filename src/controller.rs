use std::ops::Shr;

/// The controller status flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Flags {
    /// If the controller is plugged in.
    pub controller_present: bool,
    /// If the controller has a mem pak.
    pub has_mempak: bool,
    /// If the controller has a rumble pak.
    pub has_rumblepak: bool,
}

impl Flags {
    /// Creates a new instance from a `u32` type, which is the raw controller status.
    /// The raw controller status is a bitfield, with the following layout:
    /// - bit 0: controller present
    /// - bit 4: has mempak
    /// - bit 8: has rumblepak
    /// add bit 1..3 for controllers 2..4.
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

    /// Returns the raw controller status.
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

/// A single frame of controller input.
/// - Mupen64 re-recordingv2 and later versions will trigger a power off/on reset when the value for the controller info is specifically set to
/// Reserved1 = 0x01 and Reserved2 = 0x01. The controller info is then cleared from being sent to the PIF RAM to avoid errors.
/// 
/// # Raw data structure
/// | 000 - 001 | 002      | 003      |
/// |-----------|----------|----------|
/// | Buttons   | Analog X | Analog Y |
/// 
/// ## Buttons
/// Buttons pressed are determined by or-ing together values for whichever of those are pressed:
/// - 0x0001 C-Right
/// - 0x0002 C-Left
/// - 0x0004 C-Down
/// - 0x0008 C-Up
/// - 0x0010 Right shoulder
/// - 0x0020 Left shoulder
/// - 0x0040 reserved
/// - 0x0080 reserved
/// - 0x0100 Digital pad right
/// - 0x0200 Digital pad left
/// - 0x0400 Digital pad down
/// - 0x0800 Digital pad up
/// - 0x1000 Start
/// - 0x2000 Z
/// - 0x4000 B
/// - 0x8000 A
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Input {
    /// Digital pad up.
    pub up_dpad: bool,
    /// Digital pad down.
    pub down_dpad: bool,
    /// Digital pad left.
    pub left_dpad: bool,
    /// Digital pad right.
    pub right_dpad: bool,
    /// Start button.
    pub start: bool,
    /// Z button.
    pub z_button: bool,
    /// A button.
    pub a_button: bool,
    /// B button.
    pub b_button: bool,
    /// Right shoulder button.
    pub right_shoulder: bool,
    /// Left shoulder button.
    pub left_shoulder: bool,
    /// C-up.
    pub up_cbutton: bool,
    /// C-down.
    pub down_cbutton: bool,
    /// C-left.
    pub left_cbutton: bool,
    /// C-right.
    pub right_cbutton: bool,
    /// Reserved.
    pub reserved_1: bool,
    /// Reserved.
    pub reserved_2: bool,

    /// Analog stick X-axis.
    pub x_axis: i8,
    /// Analog stick Y-axis.
    pub y_axis: i8,
}

impl From<u32> for Input {
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

impl From<Input> for u32 {
    fn from(input: Input) -> Self {
        let mut value = 0;

        value |= input.right_dpad as u32;
        value |= (input.left_dpad as u32) << 1;
        value |= (input.down_dpad as u32) << 2;
        value |= (input.up_dpad as u32) << 3;
        value |= (input.start as u32) << 4;
        value |= (input.z_button as u32) << 5;
        value |= (input.b_button as u32) << 6;
        value |= (input.a_button as u32) << 7;
        value |= (input.right_cbutton as u32) << 8;
        value |= (input.left_cbutton as u32) << 9;
        value |= (input.down_cbutton as u32) << 10;
        value |= (input.up_cbutton as u32) << 11;
        value |= (input.right_shoulder as u32) << 12;
        value |= (input.left_shoulder as u32) << 13;
        value |= (input.reserved_1 as u32) << 14;
        value |= (input.reserved_2 as u32) << 15;
        value |= ((input.x_axis as u32) & 0xFF) << 16;
        value |= ((input.y_axis as u32) & 0xFF) << 24;

        value
    }
}
