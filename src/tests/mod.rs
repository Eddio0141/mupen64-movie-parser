use crate::m64::M64;

#[test]
fn test_files_parse() {
    let files = [include_bytes!("1 kick 2 boxes.m64")];

    for file in files {
        M64::from_u8_array(file).unwrap();
    }
}
