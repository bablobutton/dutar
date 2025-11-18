use crate::utils::{extract_metadata, for_each_subdir};
use dirs::{audio_dir, home_dir};
use infer::get_from_path;
use log::{debug, error};
use std::fs;
use std::path::PathBuf;

pub struct SongQueue {
    queue: Vec<Song>,
    current_idx: usize,
}

#[derive(Debug)]
pub struct Song {
    pub path: PathBuf,
    pub metadata: Option<Metadata>,
}

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
}

impl SongQueue {
    // pub fn remove() {
    //     todo!();
    // }
    //
    // pub fn clear(&mut self) {
    //     self.current_idx = 0;
    //     self.queue.clear()
    // }

    pub fn advance(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        self.current_idx += 1;
        // if no more songs, loop back to first song.
        // we'll distinguish between stop and loop back
        // later when we have that feature where user can toggle looping
        self.current_idx %= self.queue.len();
    }

    pub fn retreat(&mut self) {
        if self.queue.is_empty() {
            return;
        }

        if self.current_idx == 0 {
            // loop forward
            self.current_idx = self.queue.len() - 1;
        } else {
            self.current_idx -= 1;
        }
    }

    pub fn get_current_queue(&self) -> &[Song] {
        &self.queue
    }

    pub fn get_current_idx(&self) -> Option<usize> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.current_idx)
        }
    }

    pub fn get_current_song(&self) -> Option<&Song> {
        let current_song = self.queue.get(self.current_idx);
        if let Some(song) = current_song {
            return Some(song);
        }
        None
    }

    pub fn new() -> Self {
        let mut queue = Vec::<Song>::new();
        let dir_path = audio_dir().unwrap_or_else(|| {
            let mut dir_path = home_dir().expect("Access to home directory");
            dir_path.push("Music");
            dir_path
        });
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path).expect("Create ~/Music directory");
            debug!("Music directory created first time : {dir_path:?}");
        }

        for_each_subdir(
            dir_path.as_path(),
            &mut |dir_entry: &fs::DirEntry| match get_from_path(dir_entry.path()) {
                Ok(tp) => {
                    if tp
                        .filter(|t| t.matcher_type() == infer::MatcherType::Audio)
                        .is_some()
                    {
                        let song = Song {
                            path: dir_entry.path(),
                            metadata: extract_metadata(dir_entry.path().as_path()),
                        };
                        queue.push(song);
                    }
                }
                Err(error) => {
                    error!("Did not get file type : {error}");
                }
            },
        );

        Self {
            queue,
            current_idx: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Song> {
        self.queue.iter()
    }
}
