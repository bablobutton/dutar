mod migrations;

use crate::{SavedState, queue::SongQueue};
use color_eyre::{Result, eyre::OptionExt};
use dirs::data_dir;
use log::{debug, error};
use rusqlite::{Connection, params};
use std::{fs, path::PathBuf, time::Duration};

pub struct DB {
    conn: Connection,
}

impl DB {
    pub fn new() -> Result<Self> {
        // Linux:   ~/.local/share/dutar/dutar.db
        // macOS:   ~/Library/Application Support/dutar/dutar.db
        // Windows: C:\Users\<user>\AppData\Roaming\dutar\dutar.db
        let mut db_path = data_dir().ok_or_eyre("Couldn't get data dir path")?;
        db_path.push("dutar");
        if !db_path.exists() {
            fs::create_dir_all(&db_path)?;
            debug!("DB directory created : {db_path:?}");
        }
        db_path.push("dutar.db");

        let mut conn = Connection::open(db_path)?;

        // Create db tables for the first time.
        // If you need to add a fresh new table, add it below.
        // If you need to change existing tables, you should ONLY use migrations (migrations.rs).
        // Do NOT add new columns here - they will conflict with migrations on fresh installs.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS saved_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                volume REAL NOT NULL DEFAULT 1.0
            );
            CREATE TABLE IF NOT EXISTS queue_songs (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL
            );
            INSERT OR IGNORE INTO saved_state (id) VALUES (1);",
        )?;

        migrations::migrate(&mut conn)?;

        Ok(DB { conn })
    }

    pub fn read_saved_state(&self) -> Result<SavedState> {
        let mut sql = self.conn.prepare(
            r"
            SELECT
                volume,
                current_song_index,
                current_duration_ms
            FROM saved_state",
        )?;
        let mut rows = sql.query([])?;
        let row = rows
            .next()?
            .ok_or_eyre("1st row of saved_data should exist")?;

        let volume: f32 = row.get(0)?;
        let current_song_index: usize = row.get(1)?;
        let current_duration: Duration = Duration::from_millis(row.get(2)?);

        let saved_state = SavedState {
            volume,
            current_song_index,
            current_duration,
        };

        debug!("Read saved_state: {:?}", saved_state);

        Ok(saved_state)
    }

    pub fn write_saved_state(&self, saved_state: &SavedState) -> Result<()> {
        debug!("Write saved_state: {:?}", saved_state);
        self.conn.execute(
            r"
        UPDATE saved_state SET
            volume = ?,
            current_song_index = ?,
            current_duration_ms = ?
        WHERE id = 1
        ",
            params![
                saved_state.volume,
                saved_state.current_song_index,
                saved_state.current_duration.as_millis() as u64,
            ],
        )?;
        Ok(())
    }

    pub fn read_queue_songs_paths(&self) -> Result<Vec<PathBuf>> {
        let mut sql = self.conn.prepare("SELECT file_path FROM queue_songs")?;
        let rows = sql.query_map([], |row| row.get::<_, String>(0))?;
        let paths: Vec<PathBuf> = rows
            .filter_map(|r| match r {
                Err(e) => {
                    error!("ERROR reading queue from DB: {e}");
                    None
                }
                Ok(s) => Some(PathBuf::from(s)),
            })
            .collect();
        debug!("Read [{}] paths from saved DB queue", paths.len());
        Ok(paths)
    }

    pub fn write_queue_songs(&mut self, song_queue: &SongQueue) -> Result<()> {
        let tx = self.conn.transaction()?;

        tx.execute("DELETE FROM queue_songs", [])?;

        {
            let mut stmt = tx.prepare("INSERT INTO queue_songs (file_path) VALUES (?)")?;
            for song in song_queue.iter() {
                stmt.execute(params![song.path.to_str()])?;
            }
        }

        tx.commit()?;

        debug!(
            "Wrote {} songs to queue_songs table",
            song_queue.iter().count()
        );
        Ok(())
    }
}
