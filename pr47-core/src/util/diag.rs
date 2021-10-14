pub mod messages;

use std::mem::replace;

use smallvec::SmallVec;

use crate::util::location::{SourceLoc, SourceRange, SingleLineRange};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DiagLevel {
    Info,
    Warn,
    Error,
    Fatal
}

pub struct DiagMark {
    pub line: u32,
    pub start_col: u32,
    pub end_col: u32,

    pub comment: Option<&'static str>
}

impl DiagMark {
    pub fn new(line: u32, start_col: u32, end_col: u32) -> Self {
        Self {
            line,
            start_col,
            end_col,
            comment: None
        }
    }

    pub fn add_comment(mut self, comment: &'static str) -> Self {
        let opt: Option<&'static str> = self.comment.replace(comment);
        assert!(opt.is_none());
        self
    }
}

impl From<SourceLoc> for DiagMark {
    fn from(loc: SourceLoc) -> Self {
        Self::new(loc.line, loc.col, loc.col + 1)
    }
}

impl From<SingleLineRange> for DiagMark {
    fn from(range: SingleLineRange) -> Self {
        Self::new(range.line, range.start_col, range.end_col)
    }
}

pub struct DiagDetail<'a> {
    pub file: &'a str,
    pub detail_id: usize,
    pub mark: DiagMark,
    pub args: SmallVec<[String; 4]>,

    pub location: SourceLoc,
}

pub struct DiagDetailBuilder<'a> {
    detail: DiagDetail<'a>
}

impl<'a> DiagDetail<'a> {
    #[must_use] pub fn builder(
        file: &'a str,
        detail_id: usize,
        mark: DiagMark
    ) -> DiagDetailBuilder<'a> {
        DiagDetailBuilder {
            detail: Self {
                file,
                detail_id,
                mark,
                args: SmallVec::new(),
                location: SourceLoc::unknown(),
            }
        }
    }
}

impl<'a> DiagDetailBuilder<'a> {
    #[must_use] pub fn add_arg(mut self, arg: impl ToString) -> Self {
        self.detail.args.push(arg.to_string());
        self
    }

    #[must_use] pub fn with_location(mut self, location: SourceLoc) -> Self {
        debug_assert!(!location.is_unknown());
        self.detail.location = location;
        self
    }
}

pub struct Diagnostic<'a> {
    pub file: &'a str,
    pub diag_id: usize,
    pub args: SmallVec<[String; 4]>,

    pub location: SourceLoc,
    pub marks: SmallVec<[DiagMark; 4]>,
    pub details: SmallVec<[DiagDetail<'a>; 1]>
}

impl<'a> Diagnostic<'a> {
    #[must_use] pub fn builder(file: &'a str, diag_id: usize) -> DiagBuilder<'a> {
        DiagBuilder {
            diag: Self {
                file,
                diag_id,
                args: SmallVec::new(),

                location: SourceLoc::unknown(),
                marks: SmallVec::new(),
                details: SmallVec::new()
            }
        }
    }
}

pub struct DiagBuilder<'a> {
    diag: Diagnostic<'a>
}

impl<'a> DiagBuilder<'a> {
    #[must_use] pub fn add_arg(mut self, arg: impl ToString) -> Self {
        self.diag.args.push(arg.to_string());
        self
    }

    #[must_use] pub fn with_location(mut self, location: SourceLoc) -> Self {
        debug_assert!(!location.is_unknown());
        self.diag.location = location;
        self
    }

    #[must_use] pub fn add_mark(mut self, mark: DiagMark) -> Self {
        self.diag.marks.push(mark);
        self
    }

    #[must_use] pub fn add_detail(mut self, detail: DiagDetail<'a>) -> Self {
        self.diag.details.push(detail);
        self
    }

    #[must_use] pub fn build(self) -> Diagnostic<'a> {
        self.diag
    }
}

pub struct DiagContext<'a> {
    diags: Vec<Diagnostic<'a>>,

    has_diag: bool,
    has_error: bool
}

impl<'a> DiagContext<'a> {
    pub fn new() -> Self {
        Self {
            diags: vec![],
            has_diag: false,
            has_error: false
        }
    }

    pub fn clear_reset(&mut self) -> Vec<Diagnostic<'a>> {
        self.has_diag = false;
        self.has_error = false;

        return replace(&mut self.diags, vec![])
    }
}
