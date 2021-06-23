pub trait SourceRange : Copy + Into<MultiLineRange> {
    fn unknown() -> Self;
    fn is_unknown(&self) -> bool;

    fn start_line(&self) -> u32;
    fn end_line(&self) -> u32;
    fn start_col(&self) -> u32;
    fn end_col(&self) -> u32;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct SourceLocation {
    pub line: u32,
    pub col: u32
}

impl SourceLocation {
    pub fn new(line: u32, col: u32) -> Self {
        debug_assert_ne!(line, u32::MAX);
        debug_assert_eq!(col, u32::MAX);
        Self {
            line, col
        }
    }
}

impl Into<MultiLineRange> for SourceLocation {
    fn into(self) -> MultiLineRange {
        let mut self_clone = self.clone();
        self_clone.col += 1;
        MultiLineRange::new(self, self_clone)
    }
}

impl SourceRange for SourceLocation {
    fn unknown() -> Self {
        Self {
            line: u32::MAX,
            col: u32::MAX
        }
    }

    fn is_unknown(&self) -> bool {
        debug_assert_eq!(self.line == u32::MAX, self.col == u32::MAX);
        self.line == u32::MAX
    }

    fn start_line(&self) -> u32 { self.line }
    fn end_line(&self) -> u32 { self.line }
    fn start_col(&self) -> u32 { self.col }
    fn end_col(&self) -> u32 { self.col + 1 }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SingleLineRange {
    pub line: u32,
    pub start_col: u32,
    pub end_col: u32
}

impl SingleLineRange {
    pub fn new(line: u32, start_col: u32, end_col: u32) -> Self {
        debug_assert_ne!(line, u32::MAX);
        debug_assert_ne!(start_col, u32::MAX);
        debug_assert_ne!(end_col, u32::MAX);
        Self {
            line, start_col, end_col
        }
    }
}

impl Into<MultiLineRange> for SingleLineRange {
    fn into(self) -> MultiLineRange {
        MultiLineRange::new(
            SourceLocation::new(self.line, self.start_col),
            SourceLocation::new(self.line, self.start_col + 1)
        )
    }
}

impl SourceRange for SingleLineRange {
    fn unknown() -> Self {
        Self {
            line: u32::MAX,
            start_col: u32::MAX,
            end_col: u32::MAX
        }
    }

    fn is_unknown(&self) -> bool {
        debug_assert_eq!(self.line == u32::MAX, self.start_col == u32::MAX);
        debug_assert_eq!(self.line == u32::MAX, self.end_col == u32::MAX);
        self.line == u32::MAX
    }

    fn start_line(&self) -> u32 {
        self.line
    }

    fn end_line(&self) -> u32 {
        self.line
    }

    fn start_col(&self) -> u32 {
        self.start_col
    }

    fn end_col(&self) -> u32 {
        self.end_col
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MultiLineRange {
    pub start: SourceLocation,
    pub end: SourceLocation
}

impl MultiLineRange {
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        debug_assert!(!start.is_unknown());
        debug_assert!(!end.is_unknown());
        Self { start, end }
    }
}

impl SourceRange for MultiLineRange {
    fn unknown() -> Self {
        Self {
            start: SourceLocation::unknown(),
            end: SourceLocation::unknown()
        }
    }

    fn is_unknown(&self) -> bool {
        debug_assert_eq!(self.start.is_unknown(), self.end.is_unknown());
        self.start.is_unknown()
    }

    fn start_line(&self) -> u32 {
        self.start.line
    }

    fn end_line(&self) -> u32 {
        self.end.line
    }

    fn start_col(&self) -> u32 {
        self.start.col
    }

    fn end_col(&self) -> u32 {
        self.end.col
    }
}
