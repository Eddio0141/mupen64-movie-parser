use crate::{controller::Input, m64::M64};

#[test]
fn test_files_parse() {
    let files = [include_bytes!("1 kick 2 boxes.m64")];

    for file in files {
        let m64 = M64::from_u8_array(file).unwrap();
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
