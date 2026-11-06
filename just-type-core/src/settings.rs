//! User preferences.

use std::{fs, io, path::Path, time::Duration};

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum DurationChoice {
    #[default]
    #[serde(rename = "15s")]
    Secs15,
    #[serde(rename = "30s")]
    Secs30,
    #[serde(rename = "60s")]
    Secs60,
}

impl DurationChoice {
    pub fn as_duration(self) -> Duration {
        match self {
            DurationChoice::Secs15 => Duration::from_secs(15),
            DurationChoice::Secs30 => Duration::from_secs(30),
            DurationChoice::Secs60 => Duration::from_secs(60),
        }
    }

    /// The next longer choice, wrapping around to the shortest.
    pub fn next(self) -> Self {
        match self {
            DurationChoice::Secs15 => DurationChoice::Secs30,
            DurationChoice::Secs30 => DurationChoice::Secs60,
            DurationChoice::Secs60 => DurationChoice::Secs15,
        }
    }

    /// The next shorter choice, wrapping around to the longest.
    pub fn previous(self) -> Self {
        match self {
            DurationChoice::Secs15 => DurationChoice::Secs60,
            DurationChoice::Secs30 => DurationChoice::Secs15,
            DurationChoice::Secs60 => DurationChoice::Secs30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub duration: DurationChoice,
    /// Scale factor applied to the whole UI, on top of the OS one.
    pub zoom: f32,
    /// Whether the caret glides between characters instead of jumping.
    pub smooth_caret: bool,
    /// How long the caret takes to glide to its next position. Lower is faster.
    pub caret_glide_ms: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            duration: DurationChoice::default(),
            zoom: 1.0,
            smooth_caret: true,
            caret_glide_ms: 100,
        }
    }
}

impl Settings {
    /// How long the caret takes to glide, or `None` when it jumps.
    pub fn caret_glide(&self) -> Option<Duration> {
        self.smooth_caret.then(|| Duration::from_millis(self.caret_glide_ms))
    }

    /// Reads the settings, falling back to the defaults when the file is missing or invalid.
    pub fn load(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_else(|error| {
                eprintln!("Ignoring invalid settings in {}: {error}", path.display());
                Self::default()
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                eprintln!("Couldn't read {}: {error}", path.display());
                Self::default()
            }
        }
    }

    /// Writes a temporary file first and then replaces the old one, so a crash mid-write
    /// can't leave the settings half written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, serde_json::to_string_pretty(self)?)?;
        fs::rename(temporary, path)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use rstest::{fixture, rstest};

    use super::*;

    /// A file path in a fresh temporary directory, removed when dropped.
    struct TempFile {
        dir: PathBuf,
        path: PathBuf,
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    /// Tests run in parallel, so each one gets its own directory.
    #[fixture]
    fn temp_file() -> TempFile {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("just-type-{}-{id}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        TempFile {
            path: dir.join("settings.json"),
            dir,
        }
    }

    mod load {
        use super::*;

        #[rstest]
        fn missing_file_gives_the_defaults(temp_file: TempFile) {
            assert_eq!(Settings::load(&temp_file.path), Settings::default());
        }

        #[rstest]
        #[case::invalid_json("{ not json")]
        #[case::unknown_duration(r#"{ "duration": "45s" }"#)]
        fn invalid_file_gives_the_defaults(#[case] json: &str, temp_file: TempFile) {
            fs::write(&temp_file.path, json).unwrap();

            assert_eq!(Settings::load(&temp_file.path), Settings::default());
        }

        #[rstest]
        fn missing_fields_keep_their_defaults(temp_file: TempFile) {
            fs::write(&temp_file.path, r#"{ "zoom": 1.5 }"#).unwrap();

            let settings = Settings::load(&temp_file.path);

            assert_eq!(settings.zoom, 1.5);
            assert_eq!(settings.duration, DurationChoice::default());
        }
    }

    mod caret_glide {
        use super::*;

        #[rstest]
        #[case::smooth(true, Some(Duration::from_millis(80)))]
        #[case::jumps(false, None)]
        fn follows_the_smooth_caret_switch(#[case] smooth_caret: bool, #[case] expected: Option<Duration>) {
            let settings = Settings {
                smooth_caret,
                caret_glide_ms: 80,
                ..Settings::default()
            };

            assert_eq!(settings.caret_glide(), expected);
        }
    }

    mod save {
        use super::*;

        #[rstest]
        fn saved_settings_load_back(temp_file: TempFile) {
            let settings = Settings {
                duration: DurationChoice::Secs60,
                zoom: 1.2,
                smooth_caret: false,
                caret_glide_ms: 50,
            };

            settings.save(&temp_file.path).unwrap();

            assert_eq!(Settings::load(&temp_file.path), settings);
        }

        #[rstest]
        fn durations_are_readable(temp_file: TempFile) {
            let settings = Settings {
                duration: DurationChoice::Secs30,
                ..Settings::default()
            };

            settings.save(&temp_file.path).unwrap();

            assert!(
                fs::read_to_string(&temp_file.path)
                    .unwrap()
                    .contains(r#""duration": "30s""#)
            );
        }
    }
}
