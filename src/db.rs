use crate::SavedState;
use color_eyre::{Result, eyre::OptionExt};
use dirs::data_dir;
use log::debug;
use rusqlite::Connection;
use std::fs;

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

        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS saved_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                volume REAL NOT NULL DEFAULT 1.0
            );
            INSERT OR IGNORE INTO saved_state (id, volume) VALUES (1, 1.0);",
        )?;

        Ok(DB { conn })
    }

    pub fn read_saved_state(&self) -> Result<SavedState> {
        let mut sql = self.conn.prepare("SELECT volume FROM saved_state")?;
        let mut rows = sql.query([])?;
        let row = rows
            .next()?
            .ok_or_eyre("1st row of saved_data should exist")?;

        let volume: f32 = row.get(0)?;

        Ok(SavedState { volume })
    }

    pub fn write_saved_state(&self, saved_state: &SavedState) -> Result<()> {
        self.conn.execute(
            "UPDATE saved_state SET volume = ? WHERE id = 1",
            [saved_state.volume],
        )?;
        Ok(())
    }
}
