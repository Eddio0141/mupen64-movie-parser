# mupen64-movie-parser
-----
[![crates.io]()]()
[![Documentation]()]()

A crate for parsing Mupen64-rerecording `.m64` files.

# Example

```rust
use mupen64_movie_parser::M64;

let m64 = include_bytes!("./tests/m64s/120 star tas (2012).m64");
let m64 = M64::from_u8_array(m64).unwrap();
assert_eq!(m64.author.as_str().trim_matches(char::from(0)),
    "MKDasher, Nahoc, sonicpacker, Bauru, Eru, Goronem, Jesus, Kyman, Mokkori, Moltov, Nothing693, pasta, SilentSlayers, Snark, and ToT");
assert_eq!(m64.description.as_str().trim_matches(char::from(0)),
    "18:08.33 saved over Rikku.");
assert_eq!(m64.rerecords, 2136942);
assert_eq!(m64.vi_frames, 290491);
```