//! Contains the M64 struct and other types used for the M64 file.
use std::{
    io::{self, Read, Write},
    str::Utf8Error,
};

use arrayvec::ArrayString;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use nom::{
    bytes::complete::{tag, take},
    combinator::{map, map_opt, map_res, verify},
    multi::many0,
    number::complete::{le_u16, le_u32, u8},
    sequence::tuple,
    Finish,
};
use nom_locate::LocatedSpan;
use strum_macros::FromRepr;
use thiserror::Error;

use crate::controller::{Flags, Input};

type Span<'a> = LocatedSpan<&'a [u8]>;

/// The M64 file.
/// Follows the format described in [this document](https://tasvideos.org/EmulatorResources/Mupen/M64).
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct M64 {
    /// Identifies the movie-savestate relationship.
    /// Also used as the recording time in unix epoch format.
    pub uid: u32,
    /// Number of vertical interrupt frames.
    pub vi_frames: u32,
    /// Number of input samples for any controllers.
    pub input_frames: u32,
    /// Rerecord count.
    pub rerecords: u32,
    /// Frames per second in vertical interrupt frames.
    pub fps: u8,
    /// The number of controllers.
    pub controller_count: u8,
    /// Movie start type.
    pub movie_start_type: MovieStartType,
    /// The controller flags.
    pub controller_flags: [Flags; 4],
    /// Internal name of the ROM used when recording, directly from the ROM.
    pub rom_internal_name: ArrayString<32>,
    /// CRC32 of the ROM used when recording, directly from the ROM.
    pub rom_crc_32: u32,
    /// Country code of the ROM used when recording, directly from the ROM.
    pub rom_country_code: u16,
    /// Name of the video plugin used when recording, direcltly from the plugin.
    pub video_plugin: ArrayString<64>,
    /// Name of the sound plugin used when recording, directly from the plugin.
    pub sound_plugin: ArrayString<64>,
    /// Name of the input plugin used when recording, directly from the plugin.
    pub input_plugin: ArrayString<64>,
    /// Name of the RSP plugin used when recording, directly from the plugin.
    pub rsp_plugin: ArrayString<64>,
    /// Author(s) of the TAS.
    pub author: ArrayString<222>,
    /// Description of the TAS.
    pub description: ArrayString<256>,

    /// The input samples.
    pub inputs: Vec<Input>,
}

impl M64 {
    /// Creates an instance of `M64` from an array of bytes.
    pub fn from_u8_array(data: &[u8]) -> Result<Self, M64ParseError> {
        let data = Span::new(data);

        // defining parsers
        let signature = tag::<_, _, nom::error::Error<_>>([0x4D, 0x36, 0x34, 0x1A]);
        let movie_start_type = map_opt(le_u16, |value| MovieStartType::from_repr(value as usize));
        let controller_flags = map_opt(le_u32, |b| Some(Flags::from_u32(b)));
        let array_string = |n: usize| {
            map_res::<_, _, _, nom::error::Error<_>, _, _, _>(take(n), |s: Span| {
                std::str::from_utf8(s.fragment())
            })
        };
        let array_string_64 = || {
            map_res::<_, _, _, nom::error::Error<_>, Utf8Error, _, _>(take(64usize), |s: Span| {
                let s = std::str::from_utf8(s.fragment())?;
                Ok(ArrayString::<64>::from(s).unwrap())
            })
        };
        let input =
            map_opt::<_, _, _, nom::error::Error<_>, _, _>(le_u32, |i: u32| Some(Input::from(i)));
        let version_verify = verify(le_u32, |version| *version == 3);
        let reserved_check =
            |bytes: usize| verify(take(bytes), |v: &Span| v.iter().all(|&b| b == 0));

        // header data size check
        if data.len() < 0x400 {
            return Err(M64ParseError::NotEnoughHeaderData(data.len()));
        }

        // input data 4 bytes alignment check
        if data.len() % 4 != 0 {
            return Err(M64ParseError::InputNot4BytesAligned(data.len() % 4));
        }

        let (
            data,
            (
                _,
                _,
                uid,
                vi_frames,
                rerecords,
                fps,
                controller_count,
                _,
                input_frames,
                movie_start_type,
                _,
                controller_flags,
                _,
                rom_internal_name,
                rom_crc_32,
                rom_country_code,
                _,
                video_plugin,
                sound_plugin,
                input_plugin,
                rsp_plugin,
                // author,
                // description,
            ),
        ) = tuple((
            signature,
            version_verify,
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
            map(array_string(32), |s| ArrayString::<32>::from(s).unwrap()),
            le_u32,
            le_u16,
            reserved_check(56),
            array_string_64(),
            array_string_64(),
            array_string_64(),
            array_string_64(),
        ))(data)
        .finish()
        .map_err(|err| match err.input.location_offset() {
            0x0 => {
                M64ParseError::InvalidFileSignature(err.input.fragment()[..4].try_into().unwrap())
            }
            0x4 => M64ParseError::InvalidVersion(u32::from_le_bytes(
                err.input.fragment()[..4].try_into().unwrap(),
            )),
            0x16 | 0x1E | 0x24 | 0xEA => {
                M64ParseError::ReservedNotZero(err.input.location_offset())
            }
            0x1C => M64ParseError::InvalidMovieStartType,
            0xC4 | 0x122 | 0x162 | 0x1A2 | 0x1E2 | 0x222 | 0x300 => {
                M64ParseError::InvalidString(err.input.location_offset())
            }
            _ => unimplemented!("{:?}", err),
        })?;

        // TAS author info
        let (data, (author, description)) = tuple((
            map(array_string(222), |s| ArrayString::<222>::from(s).unwrap()),
            map(array_string(256), |s| ArrayString::<256>::from(s).unwrap()),
        ))(data)
        .finish()
        .map_err(|err| M64ParseError::InvalidString(err.input.location_offset()))?;

        // getting input data
        let (_, inputs): (_, _) = many0(input)(data).unwrap();

        Ok(M64 {
            uid,
            vi_frames,
            rerecords,
            fps,
            controller_count,
            input_frames,
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

    /// Creates an instance of `M64` from a given reader.
    pub fn read_m64<R>(mut reader: R) -> Result<Self, M64ParseError>
    where
        R: Read,
    {
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        Self::from_u8_array(&data)
    }

    /// Writes the `M64` instance to a given writer.
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
        writer.write_all(&self.vi_frames.to_le_bytes())?;
        // rerecord count
        writer.write_all(&self.rerecords.to_le_bytes())?;
        // fps
        writer.write_all(&self.fps.to_le_bytes())?;
        // controller count
        writer.write_all(&self.controller_count.to_le_bytes())?;
        // reserved
        writer.write_all(&[0; 2])?;
        // input frame count
        writer.write_all(&self.input_frames.to_le_bytes())?;
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

    /// Returns the recording time of the movie in unix epoch format, from the M64 uid.
    pub fn recording_time(&self) -> LocalResult<DateTime<Utc>> {
        Utc.timestamp_opt(self.uid as i64, 0)
    }
}

/// All possible M64 parsing errors.
#[derive(Debug, Error)]
pub enum M64ParseError {
    /// File signature didn't match.
    #[error("Invalid file signature, expected `[4D 36 34 1A]`, got `{0:X?}`")]
    InvalidFileSignature([u8; 4]),
    /// File version number wasn't 3.
    #[error("Invalid version, expected `3`, got `{0}`")]
    InvalidVersion(u32),
    /// Reserved bytes weren't zero.
    #[error("Reserved data is not all zero at offset 0x{0:X?}")]
    ReservedNotZero(usize),
    /// The header data was smaller than 1024 bytes.
    #[error("Not enough header data, expected 1024 bytes, got {0} bytes")]
    NotEnoughHeaderData(usize),
    /// The input data wasn't 4 bytes aligned.
    #[error("Input data is not 4 bytes aligned, final input data size is {0} bytes")]
    InputNot4BytesAligned(usize),
    /// Invalid movie start type.
    #[error("Invalid movie start type")]
    InvalidMovieStartType,
    /// Invalid UTF-8 string.
    #[error("Invalid UTF-8 string at offset 0x{0:X?}")]
    InvalidString(usize),
    /// Io error.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// All possible movie start types.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, FromRepr)]
pub enum MovieStartType {
    /// Movie begins from snapshot.
    /// - The snapshot will be loaded from an external file with the movie filename with the `st` extension.
    SnapShot = 1,
    /// Movie begins from power on.
    PowerOn = 2,
    /// Movie begins from EEPROM.
    Eeprom = 4,
}

impl Default for MovieStartType {
    fn default() -> Self {
        MovieStartType::PowerOn
    }
}
