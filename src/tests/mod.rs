use crate::m64::M64;

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
