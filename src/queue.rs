use crate::utils::{extract_metadata, for_each_file};
use color_eyre::{Result, eyre::WrapErr, eyre::eyre};
use dirs::{audio_dir, home_dir};
use log::{debug, error, info};
use std::fs;
use std::path::{Path, PathBuf};

pub struct SongQueue {
    queue: Vec<Song>,
    current_idx: usize,
    filtered_indices: Option<Vec<usize>>,
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
    fn new(queue: Vec<Song>) -> Self {
        Self {
            queue,
            current_idx: 0,
            filtered_indices: None,
        }
    }

    pub fn advance(&mut self) -> Result<()> {
        self.step(true)
    }

    pub fn retreat(&mut self) -> Result<()> {
        self.step(false)
    }

    fn step(&mut self, forward: bool) -> Result<()> {
        if self.queue.is_empty() {
            return Err(eyre!("Queue is empty!"));
        }
        let (len, pos) = match &self.filtered_indices {
            Some(indices) => (
                indices.len(),
                indices.iter().position(|&i| i == self.current_idx),
            ),
            None => (self.queue.len(), Some(self.current_idx)),
        };
        if len == 0 {
            return Ok(());
        }
        let next = match (pos, forward) {
            (Some(p), true) => (p + 1) % len,
            (Some(p), false) => (p + len - 1) % len,
            (None, true) => 0,
            (None, false) => len - 1,
        };
        self.current_idx = self.filtered_indices.as_ref().map_or(next, |v| v[next]);
        Ok(())
    }

    pub fn get_current_idx(&self) -> Option<usize> {
        (!self.queue.is_empty()).then_some(self.current_idx)
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
        let removed = self.current_idx;
        if removed >= self.queue.len() {
            return;
        }
        self.queue.remove(removed);
        self.current_idx = removed % self.queue.len().max(1);
        if let Some(indices) = &mut self.filtered_indices {
            indices.retain(|&i| i != removed);
            indices
                .iter_mut()
                .filter(|i| **i > removed)
                .for_each(|i| *i -= 1);
            if let Some(&i) = indices.iter().find(|&&i| i >= removed).or(indices.first()) {
                self.current_idx = i;
            }
        }
    }

    // When no queue is saved persistently to DB, dutar will scan default Music dir
    // and use that as a queue.
    // It's okay for this function to return empty queue.
    fn default_queue() -> Self {
        let dir_path = audio_dir()
            .unwrap_or_else(|| home_dir().expect("Access to home directory").join("Music"));
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path).expect("Create ~/Music directory");
            debug!("Music directory created first time : {dir_path:?}");
        }

        let songs = Self::load_songs_from_path(&dir_path).unwrap_or_else(|e| {
            error!(
                "Error loading songs from [{}], returning empty queue, error: {e}",
                dir_path.display()
            );
            Vec::new()
        });
        Self::new(songs)
    }

    pub fn restore_or_default(saved_queue_paths: Result<Vec<PathBuf>>) -> Self {
        let queue_paths = match saved_queue_paths {
            Ok(paths) if paths.is_empty() => {
                info!("Empty saved queue, reading from default dir");
                return Self::default_queue();
            }
            Ok(paths) => paths,
            Err(e) => {
                error!("Failed to read queue paths: {e}");
                return Self::default_queue();
            }
        };

        Self::new(
            queue_paths
                .iter()
                .flat_map(|p| Self::load_songs_from_path(p).unwrap_or_default())
                .collect(),
        )
    }

    // Intent of this function is when user opens a specific dir for songs.
    // I.e. it's okay for this function to return error when no songs were found under
    // provided path.
    pub fn open_path(path_str: &str) -> Result<Self> {
        let path = std::path::absolute(path_str).wrap_err("Error resolving path")?;
        let songs =
            Self::load_songs_from_path(&path).wrap_err(format!("Error opening {path_str}"))?;
        Ok(Self::new(songs))
    }

    fn load_songs_from_path(path: &Path) -> Result<Vec<Song>> {
        if !path.exists() {
            return Err(eyre!("Path does not exist: {}", path.display()));
        }

        let mut songs = Vec::new();
        if path.is_dir() {
            for_each_file(path, &mut |file| songs.extend(Self::load_song(file)));
        } else {
            songs.extend(Self::load_song(path));
        }

        if songs.is_empty() {
            return Err(eyre!("No songs found"));
        }
        Ok(songs)
    }

    fn load_song(path: &Path) -> Option<Song> {
        match infer::get_from_path(path) {
            Ok(Some(t)) if t.matcher_type() == infer::MatcherType::Audio => {
                debug!("Found audio file: {}", path.display());
                Some(Song {
                    path: path.to_path_buf(),
                    metadata: extract_metadata(path),
                })
            }
            Ok(_) => None,
            Err(e) => {
                error!("Did not get file type of {}: {e}", path.display());
                None
            }
        }
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
        let matches = |s: &str| s.to_lowercase().contains(&query);
        let indices =
            self.queue
                .iter()
                .enumerate()
                .filter(|(_, song)| {
                    song.metadata.as_ref().is_some_and(|m| {
                        matches(&m.title) || matches(&m.artist) || matches(&m.album)
                    }) || song
                        .path
                        .file_name()
                        .is_some_and(|n| matches(&n.to_string_lossy()))
                })
                .map(|(idx, _)| idx)
                .collect();

        self.filtered_indices = Some(indices);
    }

    pub fn get_display_songs(&self) -> Vec<(usize, &Song)> {
        match &self.filtered_indices {
            Some(indices) => indices.iter().map(|&i| (i, &self.queue[i])).collect(),
            None => self.queue.iter().enumerate().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue(names: &[&str]) -> SongQueue {
        SongQueue::new(
            names
                .iter()
                .map(|n| Song {
                    path: PathBuf::from(n),
                    metadata: None,
                })
                .collect(),
        )
    }

    #[test]
    fn step_wraps_within_filter() {
        let mut q = queue(&["a1", "b", "a2", "c", "a3"]);
        q.set_filter("a");
        q.advance().unwrap();
        assert_eq!(q.current_idx, 2);
        q.retreat().unwrap();
        q.retreat().unwrap();
        assert_eq!(q.current_idx, 4);
        q.advance().unwrap();
        assert_eq!(q.current_idx, 0);
    }

    #[test]
    fn remove_keeps_filter_consistent() {
        let mut q = queue(&["a1", "b", "a2", "c", "a3"]);
        q.set_filter("a");
        q.set_current_song_idx(2);
        q.remove_current_song();
        assert_eq!(q.filtered_indices, Some(vec![0, 3]));
        assert_eq!(q.get_current_song().unwrap().path, PathBuf::from("a3"));
        q.remove_current_song();
        assert_eq!(q.get_current_song().unwrap().path, PathBuf::from("a1"));
    }
}
