//! Contains the M64 struct and other types used for the M64 file.
use std::io::{self, Read, Write};

use arrayvec::ArrayString;
use chrono::{DateTime, LocalResult, TimeZone, Utc};
use strum_macros::FromRepr;
use thiserror::Error;

use crate::{
    controller::{Flags, Input},
    parser,
};

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
        Ok(parser::m64_from_u8(data).unwrap().1)
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
