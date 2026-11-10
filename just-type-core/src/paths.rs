//! Cross platform data files paths:
//!
//! - Linux:   `$XDG_CONFIG_HOME/just-type` and `$XDG_DATA_HOME/just-type`.
//! - macOS:   `~/Library/Application Support/com.arbywrk.JustType`
//! - Windows: `%APPDATA%\arbywrk\JustType\config` and `%APPDATA%\arbywrk\JustType\data`
//! - Debug builds: `data/config` and `data/data` next to the binary, e.g. `target/debug/data`.
//!
//! `JUST_TYPE_CONFIG_HOME` and `JUST_TYPE_DATA_HOME` override these in any build.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

pub struct GamePaths {
    pub settings: Option<PathBuf>,
    pub database: Option<PathBuf>,
}

impl GamePaths {
    /// Finds the app's directories and creates them if needed.
    ///
    /// If the directory can't be found or created, the app runs without that file.
    pub fn locate_or_create() -> Self {
        let config = dir("JUST_TYPE_CONFIG_HOME", "config", ProjectDirs::config_dir);
        let data = dir("JUST_TYPE_DATA_HOME", "data", ProjectDirs::data_dir);

        Self {
            settings: config.and_then(create).map(|dir| dir.join("settings.json")),
            database: data.and_then(create).map(|dir| dir.join("history.sqlite3")),
        }
    }
}

/// The directory `variable` names, or else this build's default: `debug_name` inside the
/// binary's `data` directory in debug builds, and the platform's directory in release builds.
fn dir(variable: &str, debug_name: &str, platform_dir: fn(&ProjectDirs) -> &Path) -> Option<PathBuf> {
    if let Some(dir) = env::var_os(variable).filter(|dir| !dir.is_empty()) {
        return Some(dir.into());
    }

    let dir = if cfg!(debug_assertions) {
        env::current_exe()
            .ok()
            .and_then(|exe| Some(exe.parent()?.join("data").join(debug_name)))
    } else {
        project_dirs().map(|dirs| platform_dir(&dirs).to_path_buf())
    };
    if dir.is_none() {
        eprintln!("Couldn't find where to save the {debug_name} files, so they won't be saved");
    }
    dir
}

fn project_dirs() -> Option<ProjectDirs> {
    if cfg!(any(target_os = "macos", windows)) {
        ProjectDirs::from("com", "arbywrk", "JustType")
    } else {
        ProjectDirs::from_path(PathBuf::from("just-type"))
    }
}

fn create(dir: PathBuf) -> Option<PathBuf> {
    match fs::create_dir_all(&dir) {
        Ok(()) => Some(dir),
        Err(error) => {
            eprintln!("Couldn't create {}: {error}", dir.display());
            None
        }
    }
}
