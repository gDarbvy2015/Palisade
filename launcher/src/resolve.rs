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
