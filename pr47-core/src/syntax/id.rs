use smallvec::SmallVec;

use crate::diag::location::{SourceLoc, SourceRange};

pub enum Identifier<'a> {
    Unqual(UnqualIdentifier<'a>),
    Qual(QualIdentifier<'a>)
}

pub struct UnqualIdentifier<'a> {
    pub id: &'a str,
    pub range: SourceRange
}

pub struct QualIdentifier<'a> {
    pub parts: SmallVec<[&'a str; 2]>,
    pub part_ranges: SmallVec<[&'a str; 2]>,
    pub colon_locs: Vec<SourceLoc>
}
