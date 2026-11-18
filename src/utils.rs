use crate::queue::Metadata;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::read_from_path;
use lofty::tag::Accessor;
use log::error;
use std::fs;
use std::path::Path;

pub fn for_each_file<F>(dir: &Path, cb: &mut F)
where
    F: FnMut(&Path),
{
    match fs::read_dir(dir) {
        Ok(entries) => {
            for path in entries.flatten().map(|e| e.path()) {
                if path.is_dir() {
                    for_each_file(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }
        Err(error) => error!("Error traversing {}: {error}", dir.display()),
    }
}

pub fn extract_metadata(path: &Path) -> Option<Metadata> {
    let tagged_file = read_from_path(path)
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
        .or_else(|| tagged_file.first_tag());

    Some(Metadata {
        title: tag
            .and_then(|t| t.title())
            .map(|s| s.into_owned())
            .unwrap_or_else(|| {
                path.file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default()
            }),
        artist: tag
            .and_then(|t| t.artist())
            .map(|s| s.into_owned())
            .unwrap_or_default(),
        album: tag
            .and_then(|t| t.album())
            .map(|s| s.into_owned())
            .unwrap_or_default(),
        duration_seconds: tagged_file.properties().duration().as_secs(),
    })
}
