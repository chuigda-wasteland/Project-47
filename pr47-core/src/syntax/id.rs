use crate::util::location::{SingleLineRange, SourceLocation};

#[derive(Debug)]
pub enum Identifier {
    Unqual(UnqualIdentifier),
    Qual(QualIdentifier)
}

#[derive(Debug)]
pub struct UnqualIdentifier {
    pub id: String,
    pub range: SingleLineRange
}

#[derive(Debug)]
pub struct QualIdentifier {
    pub parts: Vec<String>,
    pub part_ranges: Vec<SingleLineRange>,
    pub colon_locs: Vec<SourceLocation>
}

