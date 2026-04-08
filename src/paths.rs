use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("MLC_CONFIG_PATH") {
        return PathBuf::from(p);
    }
    dirs::config_dir()
        .expect("XDG config dir is required")
        .join("mailing-list-cli")
        .join("config.toml")
}

pub fn db_path() -> PathBuf {
    if let Ok(p) = std::env::var("MLC_DB_PATH") {
        return PathBuf::from(p);
    }
    dirs::data_local_dir()
        .expect("XDG data dir is required")
        .join("mailing-list-cli")
        .join("state.db")
}

#[allow(dead_code)]
pub fn cache_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MLC_CACHE_DIR") {
        return PathBuf::from(p);
    }
    dirs::cache_dir()
        .expect("XDG cache dir is required")
        .join("mailing-list-cli")
}

#[allow(dead_code)]
pub fn audit_log_path() -> PathBuf {
    db_path().parent().unwrap().join("audit.log")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_respects_env_override() {
        // SAFETY: tests are single-threaded for env mutation
        unsafe {
            std::env::set_var("MLC_CONFIG_PATH", "/tmp/test-config.toml");
        }
        assert_eq!(config_path(), PathBuf::from("/tmp/test-config.toml"));
        unsafe {
            std::env::remove_var("MLC_CONFIG_PATH");
        }
    }

    #[test]
    fn db_path_respects_env_override() {
        unsafe {
            std::env::set_var("MLC_DB_PATH", "/tmp/test-state.db");
        }
        assert_eq!(db_path(), PathBuf::from("/tmp/test-state.db"));
        unsafe {
            std::env::remove_var("MLC_DB_PATH");
        }
    }

    #[test]
    fn audit_log_is_sibling_of_db() {
        unsafe {
            std::env::set_var("MLC_DB_PATH", "/tmp/foo/state.db");
        }
        assert_eq!(audit_log_path(), PathBuf::from("/tmp/foo/audit.log"));
        unsafe {
            std::env::remove_var("MLC_DB_PATH");
        }
    }
}
