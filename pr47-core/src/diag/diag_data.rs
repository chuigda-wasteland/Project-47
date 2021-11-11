#![allow(non_upper_case_globals)]

// notes
pub const note_commence_placeholder: u32 = 1000;

// errors
pub const err_commence_placeholder: u32 = 2000;
pub const err_unexpected_control_char_0: u32 = 2001;
pub const err_unclosed_string_literal: u32 = 2002;
pub const err_reserved_identifier_0: u32 = 2003;
pub const err_reserved_symbol_0: u32 = 2004;
pub const err_expected_token_0_got_1: u32 = 2005;
pub const err_unexpected_eoi: u32 = 2006;
pub const err_import_decl_disallow_attr: u32 = 2007;
pub const err_export_decl_disallow_attr: u32 = 2008;
pub const err_expected_any_of_0_got_1: u32 = 2009;
pub const err_no_top_level_var_decl: u32 = 2010;

// warnings
pub const warn_commence_placeholder: u32 = 4000;
pub const warn_space_character_0_ignored: u32 = 4001;
pub const warn_underscored_id_reserved: u32 = 4002;

pub const fn is_error(code: u32) -> bool {
    code >= err_commence_placeholder && code < warn_commence_placeholder
}

pub const fn diag_message(code: u32) -> &'static str {
    if code > warn_commence_placeholder {
        match code {
            warn_space_character_0_ignored => "unicode space character '?0' ignored",
            warn_underscored_id_reserved =>
                "identifiers starting with underscore (`_`) are considered special",
            _ => "INVALID_ERROR_CODE"
        }
    } else if code > err_commence_placeholder {
        match code {
            err_unexpected_control_char_0 => "unexpected control character '?0'",
            err_unclosed_string_literal => "unclosed string literal",
            err_reserved_identifier_0 => "unexpected use of reserved identifier `?0`",
            err_reserved_symbol_0 => "unexpected use of reserved symbol `?0`",
            err_expected_token_0_got_1 => "expected ?0, got ?1",
            err_unexpected_eoi => "unexpected end of input",
            err_import_decl_disallow_attr => "cannot add attribute to `import`s",
            err_export_decl_disallow_attr => "cannot add attribute to `export`s",
            err_expected_any_of_0_got_1 => "expected any of ?0, got ?1",
            err_no_top_level_var_decl => "variable declaration cannot appear at top level",
            _ => "INVALID_ERROR_CODE"
        }
    } else /* if code > note_commence_placeholder */ {
        match code {
            _ => "INVALID_ERROR_CODE"
        }
    }
}
