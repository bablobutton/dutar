use crate::utils::for_each_subdir;
use dirs::{audio_dir, home_dir};
use infer::get_from_path;
use log::{debug, error};
use std::collections::LinkedList;
use std::fs;
use std::path::{Path, PathBuf};

pub struct SongQueue {
    list: LinkedList<Song>,
}

struct Song {
    path: PathBuf,
}

impl SongQueue {
    pub fn push_back() {
        todo!();
    }
    pub fn pop_front() {
        todo!();
    }
    pub fn clear() {
        todo!();
    }
}

impl Default for SongQueue {
    /// Initialize with all songs in the default directory.
    ///  for each file in <user's auido dir>/dutar/ (create such dir if doesn't exist)
    ///      if valid audio file, create Song object and initialize with the file
    ///      push the Song to the list
    ///
    /// User's directory per platform:
    /// | Platform | Value                  | Example                    |
    /// |----------|------------------------|----------------------------|
    /// | Linux    | XDG_MUSIC_DIR/dutar    | /home/alice/Music/dutar    |
    /// | macOS    | $HOME/Music/dutar      | /Users/Alice/Music/dutar   |
    /// | Windows  | {FOLDERID_Music}/dutar | C:\Users\Alice\Music/dutar |
    fn default() -> Self {
        let mut list = LinkedList::<Song>::new();
        let mut dir_path = audio_dir().unwrap_or_else(|| {
            let mut dir_path = home_dir().expect("Access to home directory");
            dir_path.push("Music");
            dir_path
        });
        dir_path.push("dutar");
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path).expect("Create dutar directory");
            debug!("dutar directory created first time : {dir_path:?}");
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
                        };
                        list.push_back(song);
                    }
                }
                Err(error) => {
                    error!("Did not get file type : {error}");
                }
            },
        );

        Self { list }
    }
}
