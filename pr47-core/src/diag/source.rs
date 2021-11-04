use std::cell::UnsafeCell;

use smallvec::SmallVec;
use unchecked_unwrap::UncheckedUnwrap;
use xjbutil::unchecked::UncheckedCellOps;

use crate::diag::location::SourceCoord;

pub struct SourceMap {
    line_offsets: SmallVec<[usize; 128]>
}

pub struct SourceManager {
    files: Vec<String>,
    file_contents: Vec<String>,
    source_maps: Vec<UnsafeCell<Option<SourceMap>>>
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            file_contents: Vec::new(),
            source_maps: Vec::new()
        }
    }

    pub fn add_file(&mut self, file_name: impl ToString, file_content: impl ToString) -> u32 {
        self.files.push(file_name.to_string());
        self.file_contents.push(file_content.to_string());
        self.source_maps.push(UnsafeCell::new(None));

        let file_id: usize = self.files.len() - 1;
        assert!(file_id <= (u32::MAX as usize));

        file_id as u32
    }

    pub fn compute_coord(&self, file_id: u32, file_offset: u32) -> (&str, SourceCoord) {
        let file_id: usize = file_id as usize;
        let file_offset: usize = file_offset as usize;

        self.maybe_compute_source_map(file_id);

        let source_map: &SourceMap = unsafe {
            self.source_maps[file_id as usize]
                .get_ref_unchecked()
                .as_ref()
                .unchecked_unwrap()
        };

        let line: usize = source_map.line_offsets
            .binary_search(&file_offset)
            .unwrap_or_else(|i| i - 1);
        let line_offset: usize = source_map.line_offsets[line];
        let col: usize = file_offset - line_offset;

        let file_content: &str = &self.file_contents[file_id];
        let next_line_offset: usize = if source_map.line_offsets.len() > line + 1 {
            source_map.line_offsets[line + 1]
        } else {
            file_content.len()
        };
        let source_line: &str = &file_content[line_offset..next_line_offset];

        (source_line, SourceCoord::new(line as u32, col as u32))
    }

    fn maybe_compute_source_map(&self, file_id: usize) {
        if let None = unsafe { self.source_maps[file_id].get_ref_unchecked() } {
            let file_content: &str = &self.file_contents[file_id as usize];
            let mut line_offsets: SmallVec<[usize; 128]> = SmallVec::new();
            line_offsets.push(0);
            for (i, c) in file_content.chars().enumerate() {
                if c == '\n' {
                    line_offsets.push(i + 1);
                }
            }

            unsafe {
                self.source_maps[file_id as usize]
                    .get_mut_ref_unchecked()
                    .replace(SourceMap { line_offsets });
            }
        }
    }
}
