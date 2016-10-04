//! Typed policy model built on top of the raw [`crate::policy::Document`].
//!
//! This layer gives meaning to the parsed keys and enforces the *declarative*
//! contract of a policy. It performs no process execution; it only validates
//! and normalizes intent.

use crate::policy::{parse, Document, ParseError};
use std::fmt;

/// How environment variables are handled for the child process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvMode {
    /// Start from an empty environment and add only `env.pass` / `env.set`.
    Clean,
    /// Inherit the parent environment, then remove `env.remove` and apply
    /// `env.set`.
    Inherit,
}

