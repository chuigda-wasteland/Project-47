use crate::util::location::{SingleLineRange, SourceLoc};

pub enum Identifier {
    Unqual(UnqualIdentifier),
    Qual(QualIdentifier)
}

pub struct UnqualIdentifier {
    pub id: String,
    pub range: SingleLineRange
}

pub struct QualIdentifier {
    pub parts: Vec<String>,
    pub part_ranges: Vec<SingleLineRange>,
    pub colon_locs: Vec<SourceLoc>
}

