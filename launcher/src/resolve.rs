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
