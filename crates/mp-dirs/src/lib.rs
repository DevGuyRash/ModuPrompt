use directories::BaseDirs;
use std::path::PathBuf;

const APP_DIR: &str = "moduprompt";

#[derive(Debug, Clone)]
pub struct ModuPromptDirs {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub state_dir: PathBuf,
}

pub fn resolve_dirs() -> ModuPromptDirs {
    let (config_dir, data_dir, runtime_dir) = match BaseDirs::new() {
        Some(base) => {
            let config_dir = base.config_dir().join(APP_DIR);
            let data_root = base.data_local_dir().join(APP_DIR);
            let runtime_dir = base
                .runtime_dir()
                .map(|dir| dir.join(APP_DIR))
                .unwrap_or_else(|| data_root.join("run"));
            (config_dir, data_root, runtime_dir)
        }
        None => {
            let (config_dir, data_dir) = fallback_dirs();
            let runtime_dir = data_dir.join("run");
            (config_dir, data_dir, runtime_dir)
        }
    };

    let state_dir = data_dir.join("state");

    ModuPromptDirs {
        config_dir,
        data_dir,
        runtime_dir,
        state_dir,
    }
}

pub fn config_dir() -> PathBuf {
    resolve_dirs().config_dir
}

pub fn data_dir() -> PathBuf {
    resolve_dirs().data_dir
}

pub fn runtime_dir() -> PathBuf {
    resolve_dirs().runtime_dir
}

pub fn state_dir() -> PathBuf {
    resolve_dirs().state_dir
}

pub fn default_db_path() -> PathBuf {
    state_dir().join("mpd.sqlite")
}

pub fn runtime_file_path() -> PathBuf {
    runtime_dir().join("daemon.json")
}

fn fallback_dirs() -> (PathBuf, PathBuf) {
    let home = home_dir();
    let base = home.join(".moduprompt");
    (base.join("config"), base.join("data"))
}

fn home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home);
    }
    PathBuf::from(".")
}
