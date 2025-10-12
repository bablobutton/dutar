use log::error;
use std::fs;
use std::path::Path;

pub fn for_each_subdir<F>(dir: &Path, cb: &mut F)
where
    F: FnMut(&fs::DirEntry),
{
    if dir.is_dir() {
        match fs::read_dir(dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        for_each_subdir(path.as_path(), cb);
                    } else {
                        cb(&entry);
                    }
                }
            }
            Err(error) => {
                error!("{error}");
            }
        }
    }
}
