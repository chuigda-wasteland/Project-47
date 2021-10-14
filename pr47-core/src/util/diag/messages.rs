#![allow(non_upper_case_globals)]

// errors
pub const err_commence_placeholder: u32 = 2000;
pub const err_unexpected_control_char_0: u32 = 2001;
pub const err_unclosed_string_literal: u32 = 2002;

// warnings
pub const warn_commence_placeholder: u32 = 4000;
pub const warn_space_character_0_ignored: u32 = 4001;

// notes
pub const note_commence_placeholder: u32 = 6000;

pub const fn diag_message(code: u32) -> &'static str {
    if code > note_commence_placeholder {
        match code {
            err_unexpected_control_char_0 => "unexpected control character '?0'",
            _ => "INVALID_ERROR_CODE"
        }
    } else if code > warn_commence_placeholder {
        match code {
            warn_space_character_0_ignored => "unicode space character '?0' ignored",
            _ => "INVALID_ERROR_CODE"
        }
    } else if code > err_commence_placeholder {
        match code {
            _ => "INVALID_ERROR_CODE"
        }
    } else {
        "INVALID_ERROR_CODE"
    }
}
