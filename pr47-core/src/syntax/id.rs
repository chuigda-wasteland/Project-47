use smallvec::SmallVec;

use crate::diag::location::{SourceLoc, SourceRange};

pub enum Identifier {
    Unqual(UnqualIdentifier),
    Qual(QualIdentifier)
}

pub struct UnqualIdentifier {
    pub id: String,
    pub range: SourceRange
}

pub struct QualIdentifier {
    pub parts: SmallVec<[String; 2]>,
    pub part_ranges: SmallVec<[SourceRange; 2]>,
    pub colon_locs: Vec<SourceLoc>
}
