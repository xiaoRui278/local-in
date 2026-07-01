use std::path::PathBuf;

pub fn get_filename(path: &PathBuf) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}
