use crate::queue::Metadata;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::read_from_path;
use lofty::tag::Accessor;
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
                error!("Error traversing directories: {error}");
            }
        }
    }
}

pub fn extract_metadata(path: &Path) -> Option<Metadata> {
    // try read fromat from path extention
    let tagged_file = read_from_path(path)
        // if couldn't read from extension, read from contents
        .or_else(|_| Probe::open(path)?.guess_file_type()?.read())
        .inspect_err(|e| {
            error!(
                "could not read metadata from {}, error: {e}",
                path.display()
            )
        })
        .ok()?;

    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())?;

    Some(Metadata {
        title: tag.title().map(|s| s.to_string()).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        }),
        artist: tag
            .artist()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Artist".to_string()),
        album: tag
            .album()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Album".to_string()),
        duration: tagged_file.properties().duration().as_secs(),
    })
}
