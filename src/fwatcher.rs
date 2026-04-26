use crate::{
    Message,
    queue::{Song, SongQueue},
};
use log::debug;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::mpsc::Sender,
    thread,
    time::Duration,
};

pub fn scanner(path_str: &String, tx: Sender<Message>, known_songs: Vec<Song>) {
    let mut known_map: HashMap<PathBuf, Song> = known_songs
        .into_iter()
        .map(|s| (s.path.clone(), s))
        .collect();

    let path = PathBuf::from(path_str);
    thread::spawn(move || {
        loop {
            let mut known_paths_set: HashSet<PathBuf> = known_map.keys().cloned().collect();

            let curr_scanned_songs = match SongQueue::load_songs_from_path(path.clone()) {
                Ok(songs) => songs,
                Err(err) => {
                    debug!("Failed to scan the {} with error {}", path.display(), err);
                    Vec::new()
                }
            };

            let curr_scanned_songs_set: HashSet<PathBuf> = curr_scanned_songs
                .iter()
                .map(|song| song.path.clone())
                .collect();

            let removed_paths: Vec<PathBuf> = known_paths_set
                .extract_if(|song_path| !curr_scanned_songs_set.contains(song_path))
                .collect();

            let mut new_songs: Vec<Song> = Vec::new();
            let mut removed_songs: Vec<Song> = Vec::new();

            for removed_path in removed_paths {
                if let Some(song) = known_map.remove(&removed_path) {
                    removed_songs.push(song);
                }
            }

            for song in curr_scanned_songs {
                match known_map.get(&song.path) {
                    None => {
                        new_songs.push(song.clone());
                        known_map.insert(song.path.clone(), song);
                    }
                    Some(old_song) => {
                        if old_song != &song {
                            known_map.insert(song.path.clone(), song);
                        }
                    }
                }
            }

            if !new_songs.is_empty()
                && let Err(err) = tx.send(Message::LiveAddNewSongs(new_songs))
            {
                debug!("LiveAddNewSongs send failed with {}", err);
            }

            if !removed_songs.is_empty()
                && let Err(err) = tx.send(Message::LiveRemoveSongs(removed_songs))
            {
                debug!("LiveRemoveSongs send failed with {}", err);
            }

            thread::sleep(Duration::from_secs(3));
        }
    });
}
