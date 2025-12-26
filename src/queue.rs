use crate::utils::{extract_metadata, for_each_subdir};
use color_eyre::{Result, eyre::WrapErr, eyre::eyre};
use dirs::{audio_dir, home_dir};
use infer::get_from_path;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;

pub struct SongQueue {
    queue: Vec<Song>,
    current_idx: usize,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub path: PathBuf,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_seconds: u64,
}

impl SongQueue {
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

    pub fn get_current_idx(&self) -> Option<usize> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.current_idx)
        }
    }

    pub fn get_current_song(&self) -> Option<&Song> {
        self.queue.get(self.current_idx)
    }

    pub fn set_current_song_idx(&mut self, idx: usize) {
        if idx < self.queue.len() {
            self.current_idx = idx;
        } else {
            error!(
                "Attempted to set song index [{idx}] bigger than queue size {}",
                self.queue.len()
            );
        }
    }

    // When no queue is saved persistently to DB, dutar will scan default Music dir
    // and use that as a queue.
    // It's okay for this function to return empty queue.
    fn default_queue() -> Self {
        let dir_path = audio_dir().unwrap_or_else(|| {
            let mut dir_path = home_dir().expect("Access to home directory");
            dir_path.push("Music");
            dir_path
        });
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path).expect("Create ~/Music directory");
            debug!("Music directory created first time : {dir_path:?}");
        }

        let songs = match Self::load_songs_from_path(dir_path.clone()) {
            Ok(songs) => songs,
            Err(e) => {
                error!(
                    "Error loading songs from [{}], returning empty queue, error: {e}",
                    dir_path.as_path().display()
                );
                Vec::<Song>::new()
            }
        };

        Self {
            queue: songs,
            current_idx: 0,
        }
    }

    pub fn restore_or_default(saved_queue_paths: Result<Vec<PathBuf>>) -> Self {
        let queue_paths = match saved_queue_paths {
            Ok(paths) => paths,
            Err(e) => {
                error!("Failed to read queue paths: {}", e);
                return Self::default_queue();
            }
        };
        if queue_paths.len() == 0 {
            info!("Empty saved queue, reading from default dir");
            return Self::default_queue();
        }

        let queue: Vec<Song> = queue_paths
            .iter()
            .map(|p| match Self::load_songs_from_path(p.clone()) {
                Ok(songs) => songs,
                Err(_) => vec![],
            })
            .flatten()
            .collect();

        Self {
            queue: queue,
            current_idx: 0,
        }
    }

    // Intent of this function is when user opens a specific dir for songs.
    // I.e. it's okay for this function to return error when no songs were found under
    // provided path.
    pub fn open_path(path_str: &String) -> Result<Self> {
        let songs = Self::load_songs_from_path(PathBuf::from(path_str))
            .wrap_err(format!("Error opening {path_str}"))?;

        Ok(Self {
            queue: songs,
            current_idx: 0,
        })
    }

    fn load_songs_from_path(path: PathBuf) -> Result<Vec<Song>> {
        if !path.exists() {
            return Err(eyre!("Path does not exist: {}", path.display()));
        }

        let mut songs = Vec::<Song>::new();
        if path.is_dir() {
            for_each_subdir(
                path.as_path(),
                &mut |dir_entry: &fs::DirEntry| match get_from_path(dir_entry.path()) {
                    Ok(tp) => {
                        if tp
                            .filter(|t| t.matcher_type() == infer::MatcherType::Audio)
                            .is_some()
                        {
                            songs.push(Song {
                                path: dir_entry.path(),
                                metadata: extract_metadata(dir_entry.path().as_path()),
                            });
                        }
                    }
                    Err(error) => {
                        error!("Did not get file type : {error}");
                    }
                },
            );
        } else {
            match get_from_path(&path) {
                Ok(tp) => {
                    if tp
                        .filter(|t| t.matcher_type() == infer::MatcherType::Audio)
                        .is_some()
                    {
                        songs.push(Song {
                            metadata: extract_metadata(&path),
                            path: path,
                        });
                    }
                }
                Err(error) => {
                    error!("Did not get file type : {error}");
                }
            }
        }

        if songs.len() == 0 {
            return Err(eyre!("No songs found"));
        }

        Ok(songs)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Song> {
        self.queue.iter()
    }
}
