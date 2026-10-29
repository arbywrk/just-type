//! User preferences.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::game::DurationChoice;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub duration: DurationChoice,
    /// Scale factor applied to the whole UI, on top of the OS one.
    pub zoom: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            duration: DurationChoice::default(),
            zoom: 1.0,
        }
    }
}

impl Settings {
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

    mod save {
        use super::*;

        #[rstest]
        fn saved_settings_load_back(temp_file: TempFile) {
            let settings = Settings {
                duration: DurationChoice::Secs60,
                zoom: 1.2,
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
