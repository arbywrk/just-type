//! The database containing the user's history.

use std::{
    fmt,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, params};

use crate::score::{Records, Score};

/// How many rounds the recent average covers.
const RECENT_ROUNDS: usize = 10;

/// The `mode` of timed tests, whose `mode_value` is their length in seconds.
///
/// This is currently hardcoded because the database was created with future
/// proofing for other mods in the future.
const TIME_MODE: &str = "time";

/// Database's migrations.
///
/// They are applied in order. The database's `user_version` counts how many have already run.
const MIGRATIONS: &[&str] = &[include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/migrations/0001_create_results.sql"
))];

#[derive(Debug)]
pub enum Error {
    Sqlite(rusqlite::Error),
    /// The database was created by a newer version of the app.
    UnknownVersion {
        found: usize,
        supported: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Sqlite(error) => error.fmt(f),
            Error::UnknownVersion { found, supported } => write!(
                f,
                "The database is at version {found}, but this version of the app only knows up to {supported}"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Error::Sqlite(error)
    }
}

pub struct Database {
    connection: Connection,
}

impl Database {
    /// Opens the database, creating it if needed, and brings its schema up to date.
    pub fn open(path: &Path) -> Result<Self, Error> {
        Self::migrated(Connection::open(path)?)
    }

    fn migrated(mut connection: Connection) -> Result<Self, Error> {
        migrate(&mut connection)?;
        Ok(Self { connection })
    }

    /// Saves a finished round and compares it with the earlier rounds of the same test.
    pub fn record(&mut self, score: &Score, finished_at: SystemTime) -> Result<Records, Error> {
        let mode_value = score.duration.as_secs();
        let duration_ms = u64::try_from(score.duration.as_millis()).unwrap_or(u64::MAX);
        let finished_at = finished_at.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let transaction = self.connection.transaction()?;
        let previous_best: Option<f64> = transaction.query_row(
            "SELECT MAX(wpm) FROM results WHERE mode = ?1 AND mode_value = ?2",
            params![TIME_MODE, mode_value],
            |row| row.get(0),
        )?;
        transaction.execute(
            "INSERT INTO results (finished_at, mode, mode_value, duration_ms, wpm, raw_wpm, accuracy)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                finished_at,
                TIME_MODE,
                mode_value,
                duration_ms,
                score.net_wpm,
                score.raw_wpm,
                score.accuracy
            ],
        )?;
        let (recent_average, recent_count) = transaction.query_row(
            "SELECT AVG(wpm), COUNT(*) FROM (
                SELECT wpm FROM results WHERE mode = ?1 AND mode_value = ?2 ORDER BY id DESC LIMIT ?3
             )",
            params![TIME_MODE, mode_value, RECENT_ROUNDS],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        transaction.commit()?;

        Ok(Records {
            personal_best: previous_best.map_or(score.net_wpm, |best| best.max(score.net_wpm)),
            new_personal_best: previous_best.is_some_and(|best| score.net_wpm > best),
            recent_average,
            recent_count,
        })
    }
}

fn migrate(connection: &mut Connection) -> Result<(), Error> {
    let version: usize = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let pending = MIGRATIONS.get(version..).ok_or(Error::UnknownVersion {
        found: version,
        supported: MIGRATIONS.len(),
    })?;
    if pending.is_empty() {
        return Ok(());
    }

    let transaction = connection.transaction()?;
    for migration in pending {
        transaction.execute_batch(migration)?;
    }
    transaction.pragma_update(None, "user_version", MIGRATIONS.len())?;
    transaction.commit()?;
    Ok(())
}
