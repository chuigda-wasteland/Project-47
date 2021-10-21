use smallvec::SmallVec;

use crate::diag::location::SourceCoord;

pub struct SourceMap {
    line_offsets: SmallVec<[usize; 128]>
}

pub struct SourceManager {
    files: Vec<String>,
    file_contents: Vec<String>,
    source_maps: Vec<Option<Box<SourceMap>>>
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
        self.source_maps.push(None);

        let file_id: usize = self.files.len() - 1;
        assert!(file_id <= (u32::MAX as usize));

        file_id as u32
    }

    pub fn compute_coord(&self, _file_id: u32, _file_offset: u32) -> (&str, SourceCoord) {
        todo!()
    }

    pub fn compute_coord_pair(
        &self,
        _file_id: u32,
        _file_offset_begin: u32,
        _file_offset_end: u32
    ) -> (&str, SourceCoord, SourceCoord) {
        todo!()
    }
}
