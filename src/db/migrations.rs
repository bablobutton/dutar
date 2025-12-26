use color_eyre::Result;
use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

// don't remove or modify any of existing migrations, only append new ones at the end.
const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up("ALTER TABLE saved_state ADD COLUMN current_song_index INTEGER NOT NULL DEFAULT 0;"),
    M::up("ALTER TABLE saved_state ADD COLUMN current_duration_ms INTEGER NOT NULL DEFAULT 0;"),
];
const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

pub fn migrate(conn: &mut Connection) -> Result<()> {
    MIGRATIONS.to_latest(conn)?;
    Ok(())
}
