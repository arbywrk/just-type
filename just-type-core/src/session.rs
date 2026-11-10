use std::{path::PathBuf, time::SystemTime};

use crate::{
    db::Database,
    paths::GamePaths,
    score::{Records, Score},
    settings::Settings,
};

pub struct Session {
    settings: Settings,
    /// Settings file may be missing.
    settings_path: Option<PathBuf>,
    /// Database file may be missing.
    history: Option<Database>,
}

impl Session {
    /// Loads the settings and opens the history, using the defaults for whatever is missing.
    pub fn open(paths: GamePaths) -> Self {
        let settings = paths.settings.as_deref().map(Settings::load).unwrap_or_default();
        let history = paths.database.as_deref().and_then(|path| {
            Database::open(path)
                .inspect_err(|error| eprintln!("Couldn't open the history at {}: {error}", path.display()))
                .ok()
        });

        Self {
            settings,
            settings_path: paths.settings,
            history,
        }
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The only way to change the settings. Applies `change`, brings the values back into
    /// range and saves them, skipping the write when nothing actually changed.
    pub fn change_settings(&mut self, change: impl FnOnce(&mut Settings)) {
        let previous = self.settings.clone();
        change(&mut self.settings);
        self.settings.normalize();

        if self.settings != previous
            && let Some(path) = &self.settings_path
            && let Err(error) = self.settings.save(path)
        {
            eprintln!("Couldn't save the settings to {}: {error}", path.display());
        }
    }

    /// Saves a finished round and compares it with the earlier ones.
    /// `None` when there is no history to compare with.
    pub fn record(&mut self, score: &Score, finished_at: SystemTime) -> Option<Records> {
        self.history
            .as_mut()?
            .record(score, finished_at)
            .inspect_err(|error| eprintln!("Couldn't save the result: {error}"))
            .ok()
    }
}
