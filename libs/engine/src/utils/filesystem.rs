use std::path::{Path, PathBuf};

// TODO: Fix this ugly solution to the current file retrieving method
pub fn cargo_path() -> PathBuf{
    let mut ancestor_path = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors();
    ancestor_path.next();
    ancestor_path.next();
    ancestor_path.next().expect("Error on full path").to_path_buf()
}
