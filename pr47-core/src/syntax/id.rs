use std::fmt::Formatter;

use crate::util::location::{SingleLineRange, SourceLocation};
use crate::util::mstring::{StringHandle, MToString, StringPool, MDisplay};

#[derive(Debug)]
pub enum Identifier {
    Unqual(UnqualIdentifier),
    Qual(QualIdentifier)
}

impl MToString for Identifier {
    fn m_to_string(&self, pool: &StringPool) -> String {
        match self {
            Identifier::Unqual(unqual) => unqual.m_to_string(pool),
            Identifier::Qual(qual) => qual.m_to_string(pool)
        }
    }
}

impl MDisplay for Identifier {
    fn m_fmt(&self, pool: &StringPool, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Unqual(unqual) => unqual.m_fmt(pool, fmt),
            Identifier::Qual(qual) => qual.m_fmt(pool, fmt)
        }
    }
}

#[derive(Debug)]
pub struct UnqualIdentifier {
    pub id: StringHandle,
    pub range: SingleLineRange
}

impl MToString for UnqualIdentifier {
    fn m_to_string(&self, pool: &StringPool) -> String {
        pool.get(self.id).to_string()
    }
}

impl MDisplay for UnqualIdentifier {
    fn m_fmt(&self, pool: &StringPool, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{}", pool.get(self.id))
    }
}

#[derive(Debug)]
pub struct QualIdentifier {
    pub parts: Vec<StringHandle>,
    pub part_ranges: Vec<SingleLineRange>,
    pub comma_locs: Vec<SourceLocation>
}

impl MToString for QualIdentifier {
    fn m_to_string(&self, pool: &StringPool) -> String {
        let mut ret: String = String::new();
        for (i, part) in self.parts.iter().enumerate() {
            ret.push_str(pool.get(*part));
            if i != self.parts.len() - 1 {
                ret.push_str("::")
            }
        }
        ret
    }
}

impl MDisplay for QualIdentifier {
    fn m_fmt(&self, pool: &StringPool, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, part) in self.parts.iter().enumerate() {
            write!(fmt, "{}", pool.get(*part))?;
            if i != self.parts.len() - 1 {
                write!(fmt, "::")?;
            }
        }
        Ok(())
    }
}
