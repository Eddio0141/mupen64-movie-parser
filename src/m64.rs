use std::str::Utf8Error;

use arrayvec::ArrayString;
use nom::{
    bytes::complete::{tag, take},
    character::complete::u8,
    combinator::{map_opt, map_res},
    number::complete::{le_u16, le_u32},
    sequence::tuple,
    Parser,
};
use strum_macros::FromRepr;
use thiserror::Error;

use crate::controller::{Flags, Input};

pub struct M64 {
    pub uid: u32,
    pub vi_frame_count: u32,
    pub input_frame_count: u32,
    pub rerecord_count: u32,
    pub fps: u8,
    pub controller_count: u8,
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

impl M64 {
    pub fn from_u8_array(data: &[u8]) -> Result<Self, M64ParseError> {
        let signature = tag::<_, _, nom::error::Error<_>>([0x4D, 0x36, 0x34, 0x1A]);
        let movie_start_type = map_res(u8, |b| MovieStartType::try_from(b));
        let controller_flags = map_opt(le_u32, |b| Some(Flags::from_u32(b)));
        let array_string = |n: usize| map_res(take(n), |s: &[u8]| std::str::from_utf8(s));
        let array_string_64 = || {
            map_res::<_, _, _, _, Utf8Error, _, _>(take(64usize), |s: &[u8]| {
                let s = std::str::from_utf8(s)?;
                Ok(ArrayString::<64>::from(s).unwrap())
            })
        };

        // TODO more errors
        let (data, (signature, version_number)) = tuple((signature, le_u32))(data).unwrap();

        // version number check
        if version_number != 3 {
            return Err(M64ParseError::InvalidVersionNumber(version_number));
        }

        let (data, (uid, vi_frame_count, rerecord_count, fps, controller_count, reserved)) =
            tuple((
                le_u32,
                le_u32,
                le_u32,
                u8,
                u8,
                take::<_, _, nom::error::Error<_>>(2usize),
            ))(data)
            .unwrap();

        // reserved should be 0
        // TODO does this need to be checked?
        if reserved != [0, 0] {
            return Err(M64ParseError::InvalidReserved {
                offset: 0x16,
                length: 2,
            });
        }

        let (data, (input_frame_count, movie_start_type, reserved)) = tuple((
            le_u32,
            movie_start_type,
            take::<_, _, nom::error::Error<_>>(2usize),
        ))(data)
        .unwrap();

        // reserved should be 0
        // TODO does this need to be checked?
        if reserved != [0, 0] {
            return Err(M64ParseError::InvalidReserved {
                offset: 0x1E,
                length: 2,
            });
        }

        let (data, (controller_flags, reserved)) = tuple((
            controller_flags,
            take::<_, _, nom::error::Error<_>>(160usize),
        ))(data)
        .unwrap();

        // reserved should be 0
        // TODO does this need to be checked?
        if reserved != [0; 160] {
            return Err(M64ParseError::InvalidReserved {
                offset: 0x24,
                length: 160,
            });
        }

        let (data, (rom_internal_name, rom_crc_32, rom_country_code, reserved)) = tuple((
            array_string(32).map(|s| ArrayString::<32>::from(s).unwrap()),
            le_u32,
            le_u16,
            take::<_, _, nom::error::Error<_>>(56usize),
        ))(data)
        .unwrap();

        // reserved should be 0
        // TODO does this need to be checked?
        if reserved != [0; 56] {
            return Err(M64ParseError::InvalidReserved {
                offset: 0xEA,
                length: 56,
            });
        }

        let (data, (video_plugin, audio_plugin, input_plugin, rsp_plugin, author, description)) =
            tuple((
                array_string_64(),
                array_string_64(),
                array_string_64(),
                array_string_64(),
                array_string(222).map(|s| ArrayString::<222>::from(s).unwrap()),
                array_string(256).map(|s| ArrayString::<256>::from(s).unwrap()),
            ))(data)
            .unwrap();

        println!("{:?}", signature);
        todo!("{:#?}", data)
    }
}

#[derive(Debug, Error)]
pub enum M64ParseError {
    #[error("Expected version number to be 3, got {0}")]
    InvalidVersionNumber(u32),
    #[error("Expected reserved variables to be 0, at offset {offset:#X} for {length} bytes")]
    InvalidReserved { offset: usize, length: usize },
}

#[derive(FromRepr)]
pub enum MovieStartType {
    SnapShot = 1,
    PowerOn = 2,
    Eeprom = 4,
}

impl TryFrom<u8> for MovieStartType {
    type Error = InvalidMovieStartType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_repr(value as usize).ok_or(InvalidMovieStartType(value))
    }
}

#[derive(Debug, Error)]
#[error("Invalid movie start type: {0}")]
pub struct InvalidMovieStartType(u8);

impl Default for MovieStartType {
    fn default() -> Self {
        MovieStartType::PowerOn
    }
}
