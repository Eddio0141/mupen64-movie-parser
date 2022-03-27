use arrayvec::ArrayString;

use crate::controller::{Flags, Input};

pub struct M64 {
    pub uid: u64,
    pub vi_frame_count: u64,
    pub input_frame_count: u64,
    pub rerecord_count: u64,
    pub fps: u8,
    pub number_of_controllers: u8,
    pub movie_start_type: MovieStartType,
    pub controller_flags: [Flags; 4],
    pub rom_internal_name: ArrayString<32>,
    pub rom_crc_32: u32,
    pub rom_country_code: u16,
    pub video_plugin: ArrayString<64>,
    pub audio_plugin: ArrayString<64>,
    pub input_plugin: ArrayString<64>,
    pub rsp_plugin: ArrayString<64>,
    pub author: ArrayString<222>,
    pub description: ArrayString<256>,

    pub inputs: Vec<Input>,
}

pub enum MovieStartType {
    SnapShot,
    PowerOn,
    Eeprom,
}
