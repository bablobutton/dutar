use crate::{
    Message,
    queue::{Song, SongQueue},
};
use log::debug;
use std::thread;
use std::{collections::HashMap, path::PathBuf};
use std::{sync::mpsc::Sender, time::Duration};

pub fn scanner(path_str: &String, tx: Sender<Message>, known_songs: Vec<Song>) {
    let mut known_map: HashMap<PathBuf, Song> = known_songs
        .into_iter()
        .map(|s| (s.path.clone(), s))
        .collect();

    let path = PathBuf::from(path_str);
    thread::spawn(move || {
        loop {
            let curr_scanned_songs = match SongQueue::load_songs_from_path(path.clone()) {
                Ok(songs) => songs,
                Err(err) => {
                    debug!("Failed to scan the {} with error {}", path.display(), err);
                    Vec::new()
                }
            };

            let mut new_songs = Vec::new();

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

            thread::sleep(Duration::from_secs(3));
        }
    });
}
