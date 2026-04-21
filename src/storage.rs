use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use crate::{app::App, model::Encounter};

pub fn load_encounter(path: impl AsRef<Path>) -> Result<Encounter, io::Error> {
    let contents = fs::read_to_string(path)?;
    let encounter = serde_json::from_str(&contents)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(encounter)
}

/// Return the XDG_DATA_HOME directory, or its default.
fn xdg_data_home() -> PathBuf {
    env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".local/share")
        })
}

/// Return the XDG_STATE_HOME directory, or its default.
fn xdg_state_home() -> PathBuf {
    env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".local/state")
        })
}

/// Serialize `encounter` to JSON and store it in `$XDG_DATA_HOME/intuitive/encounters` or optionally at `path`. The functions returs the path upon succesfull storage.
pub fn store_encounter(
    encounter: &Encounter,
    path: Option<impl AsRef<Path>>,
) -> Result<PathBuf, io::Error> {
    let data = serde_json::to_string_pretty(encounter)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let path = if let Some(path) = path {
        path.as_ref().to_path_buf()
    } else {
        xdg_data_home()
            .join("intuitive/encounters")
            .join(format!("encounter-{}.json", encounter.name))
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, data)?;
    Ok(path)
}

/// Store the app state to `$XDG_STATE_HOME/intuitive/state.json`
pub fn store_state(state: &App) -> Result<PathBuf, io::Error> {
    let data = serde_json::to_string_pretty(state)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let path = xdg_state_home().join("intuitive/state.json");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, data)?;
    Ok(path)
}

pub fn load_state() -> Result<Option<App>, io::Error> {
    let path = xdg_state_home().join("intuitive/state.json");

    if fs::exists(&path)? {
        let data = fs::read_to_string(path)?;
        Ok(Some(serde_json::from_str(&data).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, e)
        })?))
    } else {
        Ok(None)
    }
}
