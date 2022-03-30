use std::{
    io::{self, Read, Write},
    str::Utf8Error,
};

use arrayvec::ArrayString;
use nom::{
    bytes::complete::{tag, take},
    combinator::{map_opt, map_res, verify},
    multi::many0,
    number::complete::{le_u16, le_u32, u8},
    sequence::tuple,
    Finish, Parser,
};
use strum_macros::FromRepr;
use thiserror::Error;

use crate::controller::{Flags, Input};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
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
        let movie_start_type =
            map_res::<_, _, _, nom::error::Error<_>, _, _, _>(le_u16, MovieStartType::try_from);
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
        let mut version_verify = verify(le_u32, |version| *version == 3);
        let reserved_check =
            |bytes: usize| verify(take(bytes), |v: &[u8]| v.iter().all(|&b| b == 0));

        // header data
        let (data, _) = signature(data).finish().map_err(|err| {
            let input = err.input;
            if input.len() < 4 {
                M64ParseError::NotEnoughData {
                    expected: 4,
                    actual: input.len(),
                }
            } else {
                M64ParseError::InvalidFileSignature(input[..4].try_into().unwrap())
            }
        })?;

        // version
        let (data, _) =
            version_verify(data)
                .finish()
                .map_err(|err: nom::error::Error<&[u8]>| {
                    let input = err.input;
                    if input.len() < 4 {
                        M64ParseError::NotEnoughData {
                            expected: 4,
                            actual: input.len(),
                        }
                    } else {
                        let input = u32::from_le_bytes(input[..4].try_into().unwrap());
                        M64ParseError::InvalidVersion(input)
                    }
                })?;

        let (
            data,
            (
                uid,
                vi_frame_count,
                rerecord_count,
                fps,
                controller_count,
                _,
                input_frame_count,
                movie_start_type,
                _,
                controller_flags,
                _,
            ),
        ) = tuple((
            le_u32,
            le_u32,
            le_u32,
            u8,
            u8,
            reserved_check(2),
            le_u32,
            movie_start_type,
            reserved_check(2),
            controller_flags,
            reserved_check(160),
        ))(data)
        .finish()
        .map_err(|err| match err.code {
            nom::error::ErrorKind::Verify => M64ParseError::ReservedNotZero,
            nom::error::ErrorKind::Eof => M64ParseError::NotEnoughData {
                expected: 4,
                actual: err.input.len(),
            },
            _ => unimplemented!(),
        })?;

        // getting rom data
        let (data, (rom_internal_name, rom_crc_32, rom_country_code, _)) = tuple((
            array_string(32).map(|s| ArrayString::<32>::from(s).unwrap()),
            le_u32,
            le_u16,
            reserved_check(56),
        ))(data)
        .unwrap();

        // getting emulator data
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

        // getting input data
        let (data, inputs): (&[u8], _) = many0(input)(data).unwrap();

        if !data.is_empty() {
            return Err(M64ParseError::NotEnoughData {
                expected: 4,
                actual: data.len(),
            });
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

    pub fn read_m64<R>(mut reader: R) -> Result<Self, M64ParseError>
    where
        R: Read,
    {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::from_u8_array(&data)
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
    #[error("Invalid file signature, expected `[4D 36 34 1A]`, got `{0:X?}`")]
    InvalidFileSignature([u8; 4]),
    #[error("Invalid version, expected `3` got `{0}`")]
    InvalidVersion(u32),
    #[error("Reserved data is not zero")]
    ReservedNotZero,
    #[error("Data input too small, expected {expected} bytes, got {actual} bytes")]
    NotEnoughData { expected: usize, actual: usize },
    #[error(transparent)]
    InvalidMovieStartType(#[from] InvalidMovieStartType),
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
    #[error(transparent)]
    Io(#[from] io::Error),
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
