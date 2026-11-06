pub struct Download {
    pub file: String,
    pub progress: String,
}

#[derive(Default)]
pub struct Downloads(pub Vec<Download>);

impl Downloads {
    pub fn progress(&mut self, file: String, progress: String) {
        self.0.push(Download { file, progress });
    }

    pub fn finish(&mut self, file: &str) {
        self.0.retain(|download| download.file != file);
    }
}
