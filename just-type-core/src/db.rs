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
    /// Used to propagate SQLite errors.
    Sqlite(rusqlite::Error),
    /// The database was created by a newer version of the app.
    UnknownVersion { found: usize, supported: usize },
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

    #[cfg(test)]
    fn open_in_memory() -> Result<Self, Error> {
        Self::migrated(Connection::open_in_memory()?)
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

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::{fixture, rstest};

    #[fixture]
    fn database() -> Database {
        Database::open_in_memory().unwrap()
    }

    mod helpers {
        use super::*;

        pub fn score(wpm: f64, secs: u64) -> Score {
            Score {
                raw_wpm: wpm,
                net_wpm: wpm,
                accuracy: 100.0,
                duration: std::time::Duration::from_secs(secs),
            }
        }

        pub fn version(database: &Database) -> usize {
            database
                .connection
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .unwrap()
        }
    }

    mod migrate {
        use super::*;

        #[rstest]
        fn new_database_gets_every_migration(database: Database) {
            assert_eq!(helpers::version(&database), MIGRATIONS.len());
        }

        #[rstest]
        fn up_to_date_database_is_left_alone(mut database: Database) {
            migrate(&mut database.connection).unwrap();

            assert_eq!(helpers::version(&database), MIGRATIONS.len());
        }

        #[rstest]
        fn newer_database_is_refused(mut database: Database) {
            database
                .connection
                .pragma_update(None, "user_version", MIGRATIONS.len() + 1)
                .unwrap();

            assert!(matches!(
                migrate(&mut database.connection),
                Err(Error::UnknownVersion { .. })
            ));
        }
    }

    mod record {
        use super::*;

        fn record(database: &mut Database, wpm: f64, secs: u64) -> Records {
            database.record(&helpers::score(wpm, secs), SystemTime::now()).unwrap()
        }

        #[rstest]
        fn first_round_sets_the_best_without_celebrating(mut database: Database) {
            let records = record(&mut database, 50.0, 15);

            assert_eq!(records.personal_best, 50.0);
            assert!(!records.new_personal_best);
        }

        #[rstest]
        #[case::faster(60.0, 60.0, true)]
        #[case::slower(40.0, 50.0, false)]
        #[case::tied(50.0, 50.0, false)]
        fn compares_with_the_previous_best(
            mut database: Database,
            #[case] wpm: f64,
            #[case] expected_best: f64,
            #[case] expected_new: bool,
        ) {
            record(&mut database, 50.0, 15);

            let records = record(&mut database, wpm, 15);

            assert_eq!(records.personal_best, expected_best);
            assert_eq!(records.new_personal_best, expected_new);
        }

        #[rstest]
        fn other_round_lengths_are_ignored(mut database: Database) {
            record(&mut database, 90.0, 60);

            let records = record(&mut database, 50.0, 15);

            assert_eq!(records.personal_best, 50.0);
            assert_eq!(records.recent_count, 1);
        }

        #[rstest]
        fn averages_only_the_latest_rounds(mut database: Database) {
            record(&mut database, 1000.0, 15);
            for _ in 0..RECENT_ROUNDS {
                record(&mut database, 40.0, 15);
            }

            let records = record(&mut database, 60.0, 15);

            assert_eq!(records.recent_count, RECENT_ROUNDS);
            assert_eq!(records.recent_average, (40.0 * 9.0 + 60.0) / 10.0);
        }
    }
}
