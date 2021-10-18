use std::collections::HashMap;

pub struct SourceMap<'a>(HashMap<&'a str, Vec<&'a str>>);

impl<'a> SourceMap<'a> {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_source(&mut self, file_name: &'a str, source: Vec<&'a str>) {
        self.0.insert(file_name, source);
    }

    pub fn get_source(&self, file_name: &str, line: usize) -> Option<&'a str> {
        self.0.get(file_name)
            .and_then(|lines: &Vec<&str>| lines.get(line - 1).map(|x: &&'a str| *x))
    }
}
