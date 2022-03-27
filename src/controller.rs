pub struct Flags {
    pub controller_present: bool,
    pub has_mempak: bool,
    pub has_rumblepak: bool,
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
