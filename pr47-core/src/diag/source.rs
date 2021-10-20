use std::collections::HashMap;

use smallvec::SmallVec;

pub struct SourceMap {
    line_offsets: SmallVec<[usize; 128]>
}

pub struct SourceManager {
    file_id_map: HashMap<String, u16>,
    file_name_map: HashMap<u16, String>,
    file_contents: HashMap<String, String>,
    cached_maps: HashMap<String, SourceMap>
}
