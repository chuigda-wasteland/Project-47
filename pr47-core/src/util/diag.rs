use crate::util::location::SourceLoc;

pub mod messages;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DiagLevel {
    Info,
    Warn,
    Error,
    Fatal
}

pub struct DiagMark {
    line: u32,
    start_col: u32,
    end_col: u32
}

pub struct DiagDetail<'a> {
    file: &'a str,
    location: SourceLoc,
    marks: Vec<DiagMark>,
    detail_id: usize,
    args: Vec<String>
}

pub struct Diagnostic<'a> {
    file: &'a str,
    location: SourceLoc,
    marks: Vec<DiagMark>,
    diag_id: usize,
    args: Vec<String>,
    details: Vec<DiagDetail<'a>>
}
