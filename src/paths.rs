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
    use std::sync::Mutex;

    // Env vars are process-global. Cargo's default test runner uses a
    // thread pool, so two tests mutating the same var in parallel race.
    // This mutex serialises every env-mutating test in this module without
    // forcing `cargo test -- --test-threads=1` on the whole suite.
    //
    // The historical fix was "always use --test-threads=1"; CI was not
    // doing that, so v0.2.0 → v0.2.1 hit the race intermittently. This
    // mutex makes the test file race-free on its own.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn config_path_respects_env_override() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: serialised via ENV_MUTEX
        unsafe {
            std::env::set_var("MLC_CONFIG_PATH", "/tmp/test-config.toml");
        }
        let result = config_path();
        unsafe {
            std::env::remove_var("MLC_CONFIG_PATH");
        }
        assert_eq!(result, PathBuf::from("/tmp/test-config.toml"));
    }

    #[test]
    fn db_path_respects_env_override() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MLC_DB_PATH", "/tmp/test-state.db");
        }
        let result = db_path();
        unsafe {
            std::env::remove_var("MLC_DB_PATH");
        }
        assert_eq!(result, PathBuf::from("/tmp/test-state.db"));
    }

    #[test]
    fn audit_log_is_sibling_of_db() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("MLC_DB_PATH", "/tmp/foo/state.db");
        }
        let result = audit_log_path();
        unsafe {
            std::env::remove_var("MLC_DB_PATH");
        }
        assert_eq!(result, PathBuf::from("/tmp/foo/audit.log"));
    }
}
