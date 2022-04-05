use chrono::{TimeZone, Utc};

use crate::{controller::Input, m64::M64};

#[test]
fn test_files_parse() {
    let files = [
        include_bytes!("./m64s/1 kick 2 boxes.m64").to_vec(),
        include_bytes!("./m64s/120 star tas (2012).m64").to_vec(),
        include_bytes!("./m64s/attempt1.m64").to_vec(),
        include_bytes!("./m64s/bitfs_noreds2.m64").to_vec(),
        include_bytes!("./m64s/super mario 64 16 star tas.m64").to_vec(),
    ];

    for file in files {
        let m64 = M64::from_u8_array(&file).unwrap();
        let mut m64_u8 = Vec::new();
        m64.write_m64(&mut m64_u8).unwrap();
        assert_eq!(m64_u8, file)
    }
}

#[test]
fn inputs_parse() {
    let inputs_raw = vec![
        0b00110111_11110110_00000000_10000000u32,
        0b10000011_01111101_00000011_00000000u32,
    ];
    let inputs = vec![
        Input {
            a_button: true,
            x_axis: -10,
            y_axis: 55,
            ..Default::default()
        },
        Input {
            left_cbutton: true,
            right_cbutton: true,
            x_axis: 125,
            y_axis: -125,
            ..Default::default()
        },
    ];

    for (input_raw, input) in inputs_raw.iter().copied().zip(inputs.iter().copied()) {
        let input_raw_parsed = Input::from(input_raw);
        assert_eq!(input_raw_parsed, input);
        let input_to_raw = u32::from(input);
        assert_eq!(input_to_raw, input_raw);
    }
}

#[test]
fn invalid_signature() {
    let file = include_bytes!("./m64s/invalid_signature.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Invalid file signature, expected [4D 36 34 1A], got [FF, FF, FF, FF]"
    );
}

#[test]
fn invalid_signature_2() {
    let file = include_bytes!("./m64s/invalid_signature_2.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Invalid file signature, expected [4D 36 34 1A], got [4D, 36, 34]"
    );
}

#[test]
fn empty_file() {
    let file = Vec::new();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Invalid file signature, expected [4D 36 34 1A], got []"
    );
}

#[test]
fn not_enough_data() {
    let file = include_bytes!("./m64s/not_enough_data.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Not enough bytes to read to make up for the Version field, requires 2 more bytes"
    );
}

#[test]
fn wrong_version() {
    let file = include_bytes!("./m64s/wrong_version.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Invalid version, expected 3, got 4"
    );
}

#[test]
fn wrong_version_not_enough_data() {
    let file = include_bytes!("./m64s/wrong_version_not_enough_data.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Not enough bytes to read to make up for the Version field, requires 3 more bytes"
    );
}

#[test]
fn invalid_reserved() {
    let file = include_bytes!("./m64s/invalid_reserved.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Reserved data is not all zero at offset 0x16"
    );
}

#[test]
fn invalid_movie_start_type() {
    let file = include_bytes!("./m64s/invalid_movie_start_type.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(format!("{}", m64.unwrap_err()), "Invalid movie start type");
}

#[test]
fn invalid_utf8() {
    let file = include_bytes!("./m64s/invalid_utf8.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Invalid UTF-8 string for field RomInternalName"
    );
}

#[test]
fn not_enough_input_data() {
    let file = include_bytes!("./m64s/not_enough_input_data.m64").to_vec();
    let m64 = M64::from_u8_array(&file);
    assert_eq!(
        format!("{}", m64.unwrap_err()),
        "Input data is not 4 bytes aligned, final input data size is 2 bytes"
    );
}

#[test]
fn recording_time_test() {
    let file = include_bytes!("./m64s/120 star tas (2012).m64").to_vec();
    let m64 = M64::from_u8_array(&file).unwrap();
    assert_eq!(m64.recording_time().unwrap(), Utc.timestamp(1272727295, 0));
}
