use std::{
    io::{self, Write},
    str::Utf8Error,
};

use arrayvec::ArrayString;
use nom::{
    bytes::complete::{tag, take},
    combinator::{map_opt, map_res},
    multi::many0,
    number::complete::{le_u16, le_u32, u8},
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
    pub sound_plugin: ArrayString<64>,
    pub input_plugin: ArrayString<64>,
    pub rsp_plugin: ArrayString<64>,
    pub author: ArrayString<222>,
    pub description: ArrayString<256>,

    pub inputs: Vec<Input>,
}

impl M64 {
    pub fn from_u8_array(data: &[u8]) -> Result<Self, M64ParseError> {
        let signature = tag::<_, _, nom::error::Error<_>>([0x4D, 0x36, 0x34, 0x1A]);
        let movie_start_type = map_res(le_u16, MovieStartType::try_from);
        let controller_flags = map_opt(le_u32, |b| Some(Flags::from_u32(b)));
        let array_string = |n: usize| map_res(take(n), std::str::from_utf8);
        let array_string_64 = || {
            map_res::<_, _, _, _, Utf8Error, _, _>(take(64usize), |s: &[u8]| {
                let s = std::str::from_utf8(s)?;
                Ok(ArrayString::<64>::from(s).unwrap())
            })
        };
        let input =
            map_opt::<_, _, _, nom::error::Error<_>, _, _>(le_u32, |i: u32| Some(Input::from(i)));

        // TODO more errors
        let (data, (_, version_number)) = tuple((signature, le_u32))(data).unwrap();

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

        // println!(
        //     "uid: {}, vi_frame_count: {}, rerecord_count: {}, fps: {}, controller_count: {}",
        //     uid, vi_frame_count, rerecord_count, fps, controller_count
        // );

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

        // println!(
        //     "input_frame_count: {}, movie_start_type: {:?}",
        //     input_frame_count, movie_start_type
        // );

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

        // println!("controller_flags: {:?}", controller_flags);

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

        // println!(
        //     "rom_internal_name: {}, rom_crc_32: {}, rom_country_code: {}",
        //     rom_internal_name, rom_crc_32, rom_country_code
        // );

        // reserved should be 0
        // TODO does this need to be checked?
        if reserved != [0; 56] {
            return Err(M64ParseError::InvalidReserved {
                offset: 0xEA,
                length: 56,
            });
        }

        let (data, (video_plugin, sound_plugin, input_plugin, rsp_plugin, author, description)) =
            tuple((
                array_string_64(),
                array_string_64(),
                array_string_64(),
                array_string_64(),
                array_string(222).map(|s| ArrayString::<222>::from(s).unwrap()),
                array_string(256).map(|s| ArrayString::<256>::from(s).unwrap()),
            ))(data)
            .unwrap();

        // println!("video_plugin: {}, audio_plugin: {}, input_plugin: {}, rsp_plugin: {}, author: {}, description, {}", video_plugin, audio_plugin, input_plugin, rsp_plugin, author, description);

        let (data, inputs): (&[u8], _) = many0(input)(data).unwrap();

        if !data.is_empty() {
            return Err(M64ParseError::InvalidRemainingInputData(data.len()));
        }

        Ok(M64 {
            uid,
            vi_frame_count,
            rerecord_count,
            fps,
            controller_count,
            input_frame_count,
            movie_start_type,
            controller_flags,
            rom_internal_name,
            rom_crc_32,
            rom_country_code,
            video_plugin,
            sound_plugin,
            input_plugin,
            rsp_plugin,
            author,
            description,
            inputs,
        })
    }

    pub fn write_m64<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        // signature
        writer.write_all(b"M64\x1A")?;
        // version number
        writer.write_all(&3u32.to_le_bytes())?;
        // uid
        writer.write_all(&self.uid.to_le_bytes())?;
        // vi frame count
        writer.write_all(&self.vi_frame_count.to_le_bytes())?;
        // rerecord count
        writer.write_all(&self.rerecord_count.to_le_bytes())?;
        // fps
        writer.write_all(&self.fps.to_le_bytes())?;
        // controller count
        writer.write_all(&self.controller_count.to_le_bytes())?;
        // reserved
        writer.write_all(&[0; 2])?;
        // input frame count
        writer.write_all(&self.input_frame_count.to_le_bytes())?;
        // movie start type
        writer.write_all(&(self.movie_start_type as u16).to_le_bytes())?;
        // reserved
        writer.write_all(&[0; 2])?;
        // controller flags
        writer.write_all(&Flags::to_u32(&self.controller_flags).to_le_bytes())?;
        // reserved
        writer.write_all(&[0; 160])?;
        // rom internal name
        writer.write_all(self.rom_internal_name.as_bytes())?;
        // rom crc 32
        writer.write_all(&self.rom_crc_32.to_le_bytes())?;
        // rom country code
        writer.write_all(&self.rom_country_code.to_le_bytes())?;
        // reserved
        writer.write_all(&[0; 56])?;
        // video plugin
        writer.write_all(self.video_plugin.as_bytes())?;
        // sound plugin
        writer.write_all(self.sound_plugin.as_bytes())?;
        // input plugin
        writer.write_all(self.input_plugin.as_bytes())?;
        // rsp plugin
        writer.write_all(self.rsp_plugin.as_bytes())?;
        // author
        writer.write_all(self.author.as_bytes())?;
        // description
        writer.write_all(self.description.as_bytes())?;

        // inputs
        for input in &self.inputs {
            writer.write_all(&u32::from(*input).to_le_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum M64ParseError {
    #[error("Expected version number to be 3, got {0}")]
    InvalidVersionNumber(u32),
    #[error("Expected reserved variables to be 0, at offset {offset:#X} for {length} bytes")]
    InvalidReserved { offset: usize, length: usize },
    #[error("Invalid remaining input data: {0} bytes, must be 4 bytes aligned")]
    InvalidRemainingInputData(usize),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, FromRepr)]
pub enum MovieStartType {
    SnapShot = 1,
    PowerOn = 2,
    Eeprom = 4,
}

impl TryFrom<u16> for MovieStartType {
    type Error = InvalidMovieStartType;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::from_repr(value as usize).ok_or(InvalidMovieStartType(value))
    }
}

#[derive(Debug, Error)]
#[error("Invalid movie start type: {0}")]
pub struct InvalidMovieStartType(u16);

impl Default for MovieStartType {
    fn default() -> Self {
        MovieStartType::PowerOn
    }
}
