use arrayvec::ArrayString;
use nom::{
    bytes::complete::*, combinator::*, error::*, multi::*, number::complete::*, sequence::*,
    IResult,
};

use crate::{controller::*, m64::*};

fn array_string<'a, const S: usize>(
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], ArrayString<S>, VerboseError<&'a [u8]>> {
    let utf8_parse = map_res(take(S), std::str::from_utf8);

    map(utf8_parse, |s| ArrayString::<S>::from(s).unwrap())
}

pub fn m64_from_u8(data: &[u8]) -> IResult<(), M64, VerboseError<&[u8]>> {
    // defining parsers
    let signature = tag([0x4D, 0x36, 0x34, 0x1A]);
    let movie_start_type = map_opt(le_u16, |value| MovieStartType::from_repr(value as usize));
    let controller_flags = map_opt(le_u32, |b| Some(Flags::from_u32(b)));
    let input = map_opt(le_u32, |i: u32| Some(Input::from(i)));
    let version_verify = verify(le_u32, |version| *version == 3);
    let reserved_check = |bytes: usize| verify(take(bytes), |v: &[u8]| v.iter().all(|&b| b == 0));

    // general header data
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
        ),
    ) = tuple((
        context("signature", signature),
        context("version", version_verify),
        context("uid", le_u32),
        context("vi_frames", le_u32),
        context("rerecords", le_u32),
        context("fps", u8),
        context("controller_count", u8),
        context("reserved_0x16", reserved_check(2)),
        context("input_frames", le_u32),
        context("movie_start_type", movie_start_type),
        context("reserved_0x1E", reserved_check(2)),
        context("controller_flags", controller_flags),
        context("reserved_0x24", reserved_check(160)),
        context("rom_internal_name", array_string::<32>()),
        context("rom_crc_32", le_u32),
        context("rom_country_code", le_u16),
        context("reserved_0xEA", reserved_check(56)),
        context("video_plugin", array_string::<64>()),
        context("sound_plugin", array_string::<64>()),
        context("input_plugin", array_string::<64>()),
        context("rsp_plugin", array_string::<64>()),
    ))(data)?;

    // TAS author info
    let (data, (author, description)) = tuple((
        context("author", array_string::<222>()),
        context("description", array_string::<256>()),
    ))(data)?;

    // getting input data
    let (_, (inputs, _)) = tuple((many0(input), context("eof", eof)))(data)?;

    Ok((
        (),
        M64 {
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
        },
    ))
}
