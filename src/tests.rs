use crate::m64::M64;

#[test]
fn test_files_parse() {
    let _m64 = M64::from_u8_array(&[0x4D, 0x36, 0x34, 0x1A]).unwrap();
}