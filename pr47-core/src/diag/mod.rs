pub mod diag_data;
pub mod location;
pub mod source;

#[cfg(feature = "compiler-pretty-diag")] pub mod prettier;

use std::mem::replace;

use smallvec::SmallVec;

use crate::diag::location::{SourceLoc, SourceRange};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum DiagLevel {
    Info,
    Warn,
    Error,
    Fatal
}

pub struct DiagMark {
    pub range: SourceRange,
    pub comment: Option<&'static str>
}

impl DiagMark {
    pub fn new(file_id: u32, offset_begin: u32, offset_end: u32) -> Self {
        Self::from(SourceRange::new(file_id, offset_begin, offset_end))
    }

    pub fn add_comment(mut self, comment: &'static str) -> Self {
        let opt: Option<&'static str> = self.comment.replace(comment);
        debug_assert!(opt.is_none());
        self
    }
}

impl From<SourceLoc> for DiagMark {
    fn from(loc: SourceLoc) -> Self {
        Self::from(SourceRange::from(loc))
    }
}

impl From<SourceRange> for DiagMark {
    fn from(range: SourceRange) -> Self {
        Self {
            range, comment: None
        }
    }
}

pub struct DiagDetail {
    pub detail_id: usize,
    pub mark: DiagMark,
    pub args: SmallVec<[String; 4]>,
}

pub struct DiagDetailBuilder {
    detail: DiagDetail
}

impl<'a> DiagDetail {
    #[must_use] pub fn builder(
        detail_id: usize,
        mark: DiagMark
    ) -> DiagDetailBuilder {
        DiagDetailBuilder {
            detail: Self {
                detail_id,
                mark,
                args: SmallVec::new(),
            }
        }
    }
}

impl<'a> DiagDetailBuilder {
    #[must_use] pub fn add_arg(mut self, arg: impl ToString) -> Self {
        self.detail.args.push(arg.to_string());
        self
    }

    #[must_use] pub fn build(self) -> DiagDetail {
        self.detail
    }
}

pub struct Diagnostic {
    pub location: SourceLoc,
    pub diag_id: u32,
    pub args: SmallVec<[String; 4]>,

    pub marks: SmallVec<[DiagMark; 2]>,
    pub details: SmallVec<[DiagDetail; 1]>
}

impl Diagnostic {
    #[must_use] pub fn builder(location: SourceLoc, diag_id: u32) -> DiagBuilder {
        DiagBuilder {
            diag: Self {
                location,
                diag_id,
                args: SmallVec::new(),

                marks: SmallVec::new(),
                details: SmallVec::new()
            }
        }
    }
}

pub struct DiagBuilder {
    diag: Diagnostic
}

impl DiagBuilder {
    #[must_use] pub fn add_arg(mut self, arg: impl ToString) -> Self {
        self.diag.args.push(arg.to_string());
        self
    }

    #[must_use] pub fn add_mark(mut self, mark: DiagMark) -> Self {
        self.diag.marks.push(mark);
        self
    }

    #[must_use] pub fn add_detail(mut self, detail: DiagDetail) -> Self {
        self.diag.details.push(detail);
        self
    }

    #[must_use] pub fn build(self) -> Diagnostic {
        self.diag
    }
}

pub struct DiagContext {
    diags: Vec<Diagnostic>,

    has_diag: bool,
    has_error: bool
}

pub struct DiagBuilderCtx<'a> {
    diag_context: &'a mut DiagContext,
    diag_builder: DiagBuilder
}

impl DiagContext {
    pub fn new() -> Self {
        Self {
            diags: vec![],
            has_diag: false,
            has_error: false
        }
    }

    pub fn diag(&mut self, location: SourceLoc, diag_id: u32) -> DiagBuilderCtx {
        let diag_builder: DiagBuilder = Diagnostic::builder(location, diag_id);
        DiagBuilderCtx {
            diag_context: self,
            diag_builder
        }
    }

    pub fn add_diag(&mut self, diag: Diagnostic) {
        if diag_data::is_error(diag.diag_id) {
            self.has_error = true;
        }
        self.has_diag = true;
        self.diags.push(diag);
    }

    pub fn has_diag(&self) -> bool { self.has_diag }

    pub fn has_error(&self) -> bool { self.has_error }

    #[must_use] pub fn clear_reset(&mut self) -> Vec<Diagnostic> {
        self.has_diag = false;
        self.has_error = false;

        return replace(&mut self.diags, vec![])
    }
}

impl<'a> DiagBuilderCtx<'a> {
    #[must_use] pub fn add_arg(mut self, arg: impl ToString) -> Self {
        self.diag_builder = self.diag_builder.add_arg(arg);
        self
    }

    #[must_use] pub fn add_mark(mut self, mark: DiagMark) -> Self {
        self.diag_builder = self.diag_builder.add_mark(mark);
        self
    }

    #[must_use] pub fn add_detail(mut self, detail: DiagDetail) -> Self {
        self.diag_builder = self.diag_builder.add_detail(detail);
        self
    }

    pub fn build(self) {
        self.diag_context.add_diag(self.diag_builder.build())
    }
}

