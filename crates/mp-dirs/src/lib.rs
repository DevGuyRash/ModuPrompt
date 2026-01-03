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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        home: Option<String>,
        userprofile: Option<String>,
    }

    impl EnvGuard {
        fn set(home: Option<&str>, userprofile: Option<&str>) -> Self {
            let home_old = env::var("HOME").ok();
            let user_old = env::var("USERPROFILE").ok();

            match home {
                Some(value) => env::set_var("HOME", value),
                None => env::remove_var("HOME"),
            }
            match userprofile {
                Some(value) => env::set_var("USERPROFILE", value),
                None => env::remove_var("USERPROFILE"),
            }

            Self {
                home: home_old,
                userprofile: user_old,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.home {
                Some(value) => env::set_var("HOME", value),
                None => env::remove_var("HOME"),
            }
            match &self.userprofile {
                Some(value) => env::set_var("USERPROFILE", value),
                None => env::remove_var("USERPROFILE"),
            }
        }
    }

    #[test]
    fn home_dir_prefers_home() {
        let _lock = env_lock().lock().expect("lock");
        let _guard = EnvGuard::set(Some("/tmp/mp_home"), Some("/tmp/mp_profile"));
        assert_eq!(home_dir(), PathBuf::from("/tmp/mp_home"));
    }

    #[test]
    fn home_dir_uses_userprofile_when_home_missing() {
        let _lock = env_lock().lock().expect("lock");
        let _guard = EnvGuard::set(None, Some("/tmp/mp_profile"));
        assert_eq!(home_dir(), PathBuf::from("/tmp/mp_profile"));
    }

    #[test]
    fn fallback_dirs_use_moduprompt_layout() {
        let _lock = env_lock().lock().expect("lock");
        let _guard = EnvGuard::set(Some("/tmp/mp_home"), None);
        let (config_dir, data_dir) = fallback_dirs();
        assert_eq!(
            config_dir,
            PathBuf::from("/tmp/mp_home/.moduprompt/config")
        );
        assert_eq!(
            data_dir,
            PathBuf::from("/tmp/mp_home/.moduprompt/data")
        );
    }

    #[test]
    fn default_paths_have_expected_suffixes() {
        let db_path = default_db_path();
        assert!(db_path.ends_with(Path::new("state").join("mpd.sqlite")));

        let runtime_path = runtime_file_path();
        assert!(runtime_path.ends_with(Path::new("daemon.json")));
    }
}
