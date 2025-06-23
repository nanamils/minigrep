use std::{fs, io::{self, Read}, path::Path};

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with('.') || s == "target")
         .unwrap_or(false)
}

pub fn is_binary(path: &Path) -> io::Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut buffer = [0; 1024];
    let n = file.read(&mut buffer)?;
    Ok(buffer[..n].contains(&0))
}