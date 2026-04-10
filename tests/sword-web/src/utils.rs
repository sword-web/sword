pub struct TempFile {
    pub path: String,
}

impl TempFile {
    pub fn with_size(size: usize) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let project_root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let file_path = format!("{project_root}/files/test_file_{timestamp}.txt");

        let content = vec![b'x'; size];
        std::fs::write(&file_path, content).expect("Failed to write test file");

        Self { path: file_path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
