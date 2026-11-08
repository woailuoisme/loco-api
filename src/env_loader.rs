//! Environment variable loader with multi-environment support.
//!
//! Priority:
//! 1. Existing host / process environment variables (never overwritten)
//! 2. `.env.<environment>` (e.g. `.env.development`, `.env.production`, `.env.test`)
//! 3. `.env` (base fallback)

use std::env;

/// Detects the current active Loco environment.
///
/// Priority:
/// 1. CLI flags (`-e <env>`, `--environment <env>`, `--environment=<env>`)
/// 2. `LOCO_ENV`
/// 3. `RAILS_ENV`
/// 4. `NODE_ENV`
/// 5. Defaults to `"development"`.
#[must_use]
pub fn detect_environment() -> String {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-e" || arg == "--environment" {
            if let Some(val) = args.next() {
                return val;
            }
        } else if let Some(val) = arg.strip_prefix("--environment=") {
            return val.to_string();
        }
    }

    env::var("LOCO_ENV")
        .or_else(|_| env::var("RAILS_ENV"))
        .or_else(|_| env::var("NODE_ENV"))
        .unwrap_or_else(|_| "development".to_string())
}

/// Loads environment variables based on the active environment.
///
/// 1. Loads `.env.<environment>` (if present) without overriding existing process variables.
/// 2. Loads `.env` (if present) as base fallback for any still unset variables.
pub fn load_env() {
    let env_name = detect_environment();
    let _ = dotenvy::from_filename(format!(".env.{env_name}"));
    let _ = dotenvy::dotenv();
}

/// Specifically loads `.env.test` followed by `.env` (used for test suites).
pub fn load_test_env() {
    let _ = dotenvy::from_filename(".env.test");
    let _ = dotenvy::dotenv();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_environment_not_empty() {
        let env = detect_environment();
        assert!(!env.is_empty());
    }
}
