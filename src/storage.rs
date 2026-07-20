use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::app::App;
use crate::model::Encounter;

const ENCOUNTER_RECORD_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterRecord {
    pub schema_version: u16,
    pub encounter: Encounter,
}

impl EncounterRecord {
    pub fn new(encounter: Encounter) -> Self {
        Self {
            schema_version: ENCOUNTER_RECORD_VERSION,
            encounter,
        }
    }
}

pub fn load_encounter(path: impl AsRef<Path>) -> Result<Encounter, io::Error> {
    let contents = fs::read_to_string(path)?;
    let record: EncounterRecord = serde_json::from_str(&contents)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if record.schema_version != ENCOUNTER_RECORD_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Unsupported encounter schema_version: {}",
                record.schema_version
            ),
        ));
    }

    Ok(record.encounter)
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
    let record = EncounterRecord::new(encounter.clone());
    let data = serde_json::to_string_pretty(&record)
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

const APP_STATE_RECORD_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateRecord {
    pub schema_version: u16,
    pub state: App,
}

impl AppStateRecord {
    pub fn new(state: App) -> Self {
        Self {
            schema_version: APP_STATE_RECORD_VERSION,
            state,
        }
    }
}

fn store_state_to(state: &App, path: impl AsRef<Path>) -> Result<PathBuf, io::Error> {
    let record = AppStateRecord::new(state.clone());
    let data = serde_json::to_string_pretty(&record)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let path = path.as_ref().to_path_buf();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, data)?;
    Ok(path)
}

fn load_state_from(path: impl AsRef<Path>) -> Result<Option<App>, io::Error> {
    let path = path.as_ref();

    if fs::exists(path)? {
        let data = fs::read_to_string(path)?;
        let record: AppStateRecord = serde_json::from_str(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        if record.schema_version != APP_STATE_RECORD_VERSION {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported state schema_version: {}",
                    record.schema_version
                ),
            ));
        }

        Ok(Some(record.state))
    } else {
        Ok(None)
    }
}

/// Store the app state to `$XDG_STATE_HOME/intuitive/state.json`
pub fn store_state(state: &App) -> Result<PathBuf, io::Error> {
    store_state_to(state, xdg_state_home().join("intuitive/state.json"))
}

pub fn load_state() -> Result<Option<App>, io::Error> {
    load_state_from(xdg_state_home().join("intuitive/state.json"))
}

#[cfg(test)]
mod tests {
    use super::{EncounterRecord, load_encounter};
    use crate::storage::Encounter;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_file_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("intuitive-{}-{}.json", name, nanos))
    }

    #[test]
    fn load_encounter_accepts_current_record_version() {
        let path = temp_file_path("encounter-v1");
        let record = EncounterRecord::new(Encounter::default());
        let json = serde_json::to_string(&record).unwrap();
        fs::write(&path, json).unwrap();

        let loaded = load_encounter(&path).unwrap();
        assert_eq!(loaded.creatures.len(), 0);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn load_encounter_rejects_unknown_record_version() {
        let path = temp_file_path("encounter-v999");
        let record = EncounterRecord {
            schema_version: 999,
            encounter: Encounter::default(),
        };
        let json = serde_json::to_string(&record).unwrap();
        fs::write(&path, json).unwrap();

        let err = load_encounter(&path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

        fs::remove_file(&path).unwrap();
    }
}

#[cfg(test)]
mod state_tests {
    use super::{App, AppStateRecord, load_state_from, store_state_to};
    use crate::model::Creature;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_file_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("intuitive-{}-{}.json", name, nanos))
    }

    #[test]
    fn load_state_accepts_current_record_version() {
        let path = temp_file_path("state-v1");
        let record = AppStateRecord::new(App::default());
        let json = serde_json::to_string(&record).unwrap();
        fs::write(&path, json).unwrap();

        let loaded = load_state_from(&path).unwrap();
        assert!(loaded.is_some());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn load_state_rejects_unknown_record_version() {
        let path = temp_file_path("state-v999");
        let record = AppStateRecord {
            schema_version: 999,
            state: App::default(),
        };
        let json = serde_json::to_string(&record).unwrap();
        fs::write(&path, json).unwrap();

        let err = load_state_from(&path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn load_state_returns_none_when_file_missing() {
        let path = temp_file_path("state-missing");
        assert!(load_state_from(&path).unwrap().is_none());
    }

    #[test]
    fn store_state_round_trips_through_load() {
        let path = temp_file_path("state-roundtrip");
        let mut app = App::default();
        app.current_encounter
            .add_creature(Creature::new_player("Alice", 10, 10, None, None, None));

        store_state_to(&app, &path).unwrap();
        let loaded = load_state_from(&path).unwrap().unwrap();

        assert_eq!(loaded.current_encounter.creatures.len(), 1);

        fs::remove_file(&path).unwrap();
    }
}
