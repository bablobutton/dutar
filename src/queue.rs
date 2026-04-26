use crate::utils::{extract_metadata, for_each_subdir};
use color_eyre::{Result, eyre::WrapErr, eyre::eyre};
use dirs::{audio_dir, home_dir};
use infer::get_from_path;
use log::{debug, error, info};
use std::fs;
use std::path::PathBuf;

pub struct SongQueue {
    pub queue: Vec<Song>,
    current_idx: usize,
    filtered_indices: Option<Vec<usize>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Song {
    pub path: PathBuf,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_seconds: u64,
}

impl SongQueue {
    pub fn advance(&mut self) -> Result<()> {
        if self.queue.is_empty() {
            return Err(eyre!("Queue is empty!"));
        }

        if let Some(indices) = &self.filtered_indices {
            if indices.is_empty() {
                return Ok(()); // Некуда переключать
            }

            if let Some(pos) = indices
                .iter()
                .position(|&real_idx| real_idx == self.current_idx)
            {
                let next_pos = (pos + 1) % indices.len();
                self.current_idx = indices[next_pos];
            } else {
                self.current_idx = indices[0];
            }
        } else {
            self.current_idx += 1;
            self.current_idx %= self.queue.len();
        }

        Ok(())
    }

    pub fn retreat(&mut self) -> Result<()> {
        if self.queue.is_empty() {
            return Err(eyre!("Queue is empty!"));
        }

        if let Some(indices) = &self.filtered_indices {
            if indices.is_empty() {
                return Ok(());
            }

            if let Some(pos) = indices
                .iter()
                .position(|&real_idx| real_idx == self.current_idx)
            {
                let prev_pos = if pos == 0 { indices.len() - 1 } else { pos - 1 };
                self.current_idx = indices[prev_pos];
            } else {
                self.current_idx = indices[indices.len() - 1];
            }
        } else {
            if self.current_idx == 0 {
                self.current_idx = self.queue.len() - 1;
            } else {
                self.current_idx -= 1;
            }
        }
        Ok(())
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

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn remove_current_song(&mut self) {
        if self.current_idx < self.queue.len() {
            let removed = self.queue.remove(self.current_idx);
            if self.queue.is_empty() {
                self.current_idx = 0;
            } else {
                self.current_idx %= self.queue.len();
            }
            let title = if let Some(metadata) = &removed.metadata {
                metadata.title.as_str()
            } else {
                "Unknown"
            };

            debug!("Removed song [{}] from queue", title);
        }
    }

    // returns a flag signaling that the currently selected song was removed
    pub fn remove_songs(&mut self, removed_songs: Vec<Song>) -> bool {
        let removed_songs_count = removed_songs.len();

        // song can be selected, but not playing
        let removed_current_song = if let Some(curr_selected_song) = self.get_current_song() {
            removed_songs.contains(curr_selected_song)
        } else {
            false
        };

        self.queue.retain(|song| !removed_songs.contains(song));

        if self.queue.is_empty() {
            self.current_idx = 0
        } else {
            self.current_idx %= self.queue.len();
        }

        debug!("Removed {} songs from the queue", removed_songs_count);

        removed_current_song
    }

    pub fn add_new_songs(&mut self, new_songs: Vec<Song>) {
        let new_song_count = new_songs.len();
        self.queue.extend(new_songs);

        debug!("Added {} songs to the queue", new_song_count);
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
            filtered_indices: None,
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
            queue,
            current_idx: 0,
            filtered_indices: None,
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
            filtered_indices: None,
        })
    }

    pub fn load_songs_from_path(path: PathBuf) -> Result<Vec<Song>> {
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
                            path,
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

    pub fn set_filter(&mut self, query: &str) {
        if query.is_empty() {
            self.filtered_indices = None;
            return;
        }

        let query = query.to_lowercase();
        let indices: Vec<usize> = self
            .queue
            .iter()
            .enumerate()
            .filter(|(_, song)| {
                // 1. Check Metadata if available
                if let Some(meta) = &song.metadata {
                    if meta.title.to_lowercase().contains(&query) {
                        return true;
                    }
                    if meta.artist.to_lowercase().contains(&query) {
                        return true;
                    }
                    if meta.album.to_lowercase().contains(&query) {
                        return true;
                    }
                }

                // 2. Fallback to filename
                if let Some(name) = song.path.file_name() {
                    if name.to_string_lossy().to_lowercase().contains(&query) {
                        return true;
                    }
                }

                false
            })
            .map(|(idx, _)| idx)
            .collect();

        self.filtered_indices = Some(indices);
    }

    pub fn get_display_songs(&self) -> Vec<(usize, &Song)> {
        match &self.filtered_indices {
            Some(indices) => indices
                .iter()
                .filter_map(|&i| self.queue.get(i).map(|song| (i, song)))
                .collect(),
            None => self.queue.iter().enumerate().collect(),
        }
    }
}
