//! Executable resolution and path-root validation.
//!
//! Resolution rules:
//! - If `program` contains a path separator (`/` on all platforms, `\\` on
//!   Windows), it is treated as a path and used directly (after canonicalizing
//!   to an absolute path).
//! - Otherwise it is a bare command name resolved against the `PATH`
//!   environment variable. On Windows, `PATHEXT` is honored so that `git`
//!   resolves to `git.exe`, etc.
//!
//! Once resolved, if the policy declares `path.root` entries, the resolved
//! absolute path must be contained within at least one of those roots. This is
//! a userspace guard, not a kernel-enforced jail.

use std::path::{Path, PathBuf};

/// Error produced while resolving an executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    /// The bare command could not be found on PATH.
    NotFound(String),
    /// The explicit path does not exist or is not a file.
    NotAFile(String),
    /// The resolved path is not under any declared `path.root`.
    OutsideRoots {
        resolved: String,
        roots: Vec<String>,
    },
    /// The filesystem could not be queried.
    Io(String),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::NotFound(name) => {
                write!(f, "executable '{name}' not found on PATH")
            }
            ResolveError::NotAFile(p) => {
                write!(f, "'{p}' does not exist or is not a regular file")
            }
            ResolveError::OutsideRoots { resolved, roots } => write!(
                f,
                "resolved executable '{resolved}' is not under any allowed path.root: {roots:?}"
            ),
            ResolveError::Io(m) => write!(f, "filesystem error during resolution: {m}"),
        }
    }
}

impl std::error::Error for ResolveError {}

fn has_separator(s: &str) -> bool {
    s.contains('/') || (cfg!(windows) && s.contains('\\'))
}

/// Return the PATHEXT-derived list of executable extensions on Windows,
/// or a single empty extension elsewhere.
fn path_extensions() -> Vec<String> {
    if cfg!(windows) {
        match std::env::var("PATHEXT") {
            Ok(v) => {
                let mut exts: Vec<String> = v
                    .split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_ascii_lowercase())
                    .collect();
                if exts.is_empty() {
                    exts.push(".exe".to_string());
                }
                // Also allow an exact match with no extension.
                exts.push(String::new());
                exts
            }
            Err(_) => vec![
                ".com".to_string(),
                ".exe".to_string(),
                ".bat".to_string(),
                ".cmd".to_string(),
                String::new(),
            ],
        }
    } else {
        vec![String::new()]
    }
}

fn is_executable_file(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(md) => md.is_file(),
        Err(_) => false,
    }
}

/// Resolve a bare command name against PATH (and PATHEXT on Windows).
fn resolve_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let exts = path_extensions();
    for dir in std::env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        for ext in &exts {
            let mut candidate = dir.join(name);
