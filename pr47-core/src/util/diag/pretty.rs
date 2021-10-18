use std::iter::Peekable;

use crate::util::diag::Diagnostic;
use crate::util::diag::messages;
use crate::util::source_map::SourceMap;
use crate::util::location::{SourceRange, SourceLoc};

pub fn prettify_diag<'a>(diag: &Diagnostic<'a>, source_map: &SourceMap<'a>) -> String {
    let SourceLoc { line, col } = diag.location;

    let mut ret: String = format!(
        "{}[{}]: {}",
        error_level(diag.diag_id),
        diag.diag_id,
        format_diag_message(messages::diag_message(diag.diag_id), &diag.args)
    );

    if diag.location.is_unknown() {
        return ret;
    }

    let line_num_width: usize = digit_count(line);

    ret.push('\n');
    for _ in 0..line_num_width {
        ret.push(' ');
    }
    ret.push_str(&format!("--> {}:{}:{}\n", diag.file, line, col));

    for _ in 0..(line_num_width + 1) {
        ret.push(' ');
    }
    ret.push_str("|\n");

    ret.push_str(&format!(
        "{} | {}\n",
        diag.location.line,
        source_map.get_source(diag.file, line as usize)
            .unwrap_or("source code not available")
    ));

    for _ in 0..(line_num_width + 1) {
        ret.push(' ');
    }
    ret.push('|');

    if diag.marks.is_empty() {
        ret.push('\n');
        return ret;
    }

    let mut current_pos: u32 = 0;
    for mark in diag.marks.iter() {
        if current_pos != mark.start_col {
            for _ in current_pos..mark.start_col {
                ret.push(' ');
            }
            for _ in mark.start_col..mark.end_col {
                ret.push('^');
            }
            current_pos = mark.end_col;
        }
    }
    ret.push('\n');

    for _ in 0..(line_num_width + 1) {
        ret.push(' ');
    }
    ret.push('|');

    for mark in diag.marks.iter() {
        if let Some(comment) = mark.comment {
            for _ in 0..mark.start_col {
                ret.push(' ');
            }
            ret.push_str(comment);
            break;
        }
    }
    ret.push('\n');

    ret
}

fn format_diag_message(template: &str, args: &[impl AsRef<str>]) -> String {
    let mut peekable: Peekable<_> = template.chars().peekable();
    let mut output: String = String::new();

    while let Some(ch /*: char*/) = peekable.next() {
        if ch == '?' {
            if let Some(next_ch /*: char*/) = peekable.peek().map(|x: &char| *x) {
                if next_ch.is_digit(10) {
                    let _ = peekable.next();
                    let idx: u32 = (next_ch as u32) - ('0' as u32);
                    output.push_str(args[idx as usize].as_ref());
                    continue;
                } else if next_ch == '?' {
                    let _ = peekable.next();
                    output.push('?');
                    continue;
                }
            }
        }

        output.push(ch);
    }

    output
}

const fn error_level(diag_id: u32) -> &'static str {
    if diag_id >= messages::warn_commence_placeholder {
        "warning"
    } else if diag_id >= messages::err_commence_placeholder {
        "error"
    } else {
        "note"
    }
}

fn digit_count<T>(mut i: T) -> usize
    where T: std::ops::DivAssign + std::cmp::PartialOrd + From<u8> + Copy
{
    let mut len: usize = 0;
    let zero = T::from(0);
    let ten = T::from(10);

    while i > zero {
        i /= ten;
        len += 1;
    }

    len
}
