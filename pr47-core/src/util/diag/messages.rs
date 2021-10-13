#![allow(non_upper_case_globals)]

use phf::phf_map;

pub const unknown_character_0: u32 = 0;

pub const fn diag_message(code: u32) -> &'static str {
    match code {
        unknown_character_0 => "unknown character ?0",
        _ => "INVALID_ERROR_CODE"
    }
}
