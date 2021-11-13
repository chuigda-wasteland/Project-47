use crate::diag::source::SourceManager;

#[derive(Clone, Copy)]
pub struct SourceCoord {
    pub line: u32,
    pub col: u32
}

impl SourceCoord {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SourceLoc {
    pub file_id: u32,
    pub offset: u32
}

impl SourceLoc {
    pub fn new(file_id: u32, offset: u32) -> Self {
        Self { file_id, offset }
    }

    pub fn unknown() -> Self {
        Self::new(u32::MAX, u32::MAX)
    }

    pub fn is_unknown(&self) -> bool {
        debug_assert_eq!(self.file_id == u32::MAX, self.offset == u32::MAX);
        self.file_id == u32::MAX
    }

    pub fn compute_coord<'b>(&self, source_mgr: &'b SourceManager) -> (&'b str, SourceCoord) {
        source_mgr.compute_coord(self.file_id, self.offset)
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SourceRange {
    file_id: u32,
    offset_begin: u32,
    offset_end: u32
}

impl SourceRange {
    pub fn new(file_id: u32, offset_begin: u32, offset_end: u32) -> Self {
        Self { file_id, offset_begin, offset_end }
    }

    pub fn from_loc_pair(left: SourceLoc, right: SourceLoc) -> Self {
        debug_assert_eq!(left.file_id, right.file_id);
        debug_assert!(left.offset <= right.offset);

        Self::new(left.file_id, left.offset, right.offset)
    }

    pub fn unknown() -> Self {
        Self::new(u32::MAX, u32::MAX, u32::MAX)
    }

    pub fn is_unknown(&self) -> bool {
        debug_assert_eq!(self.file_id == u32::MAX, self.offset_begin == u32::MAX);
        debug_assert_eq!(self.offset_begin == u32::MAX, self.offset_end == u32::MAX);
        self.file_id == u32::MAX
    }

    pub fn left(&self) -> SourceLoc {
        SourceLoc::new(self.file_id, self.offset_begin)
    }

    pub fn right(&self) -> SourceLoc {
        SourceLoc::new(self.file_id, self.offset_end)
    }

    pub fn compute_coord_pair<'b>(
        &self,
        source_mgr: &'b SourceManager
    ) -> ((&'b str, SourceCoord), (&'b str, SourceCoord)) {
        let (begin_line, begin_coord): (&'b str, SourceCoord)
            = source_mgr.compute_coord(self.file_id, self.offset_begin);
        let (end_line, end_coord): (&'b str, SourceCoord)
            = source_mgr.compute_coord(self.file_id, self.offset_end);

        ((begin_line, begin_coord), (end_line, end_coord))
    }
}

impl From<SourceLoc> for SourceRange {
    fn from(location: SourceLoc) -> Self {
        Self::new(location.file_id, location.offset, location.offset)
    }
}
