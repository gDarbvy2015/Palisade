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

/// Declared network intent. Palisade cannot enforce network access in
/// userspace; this field documents intent and is surfaced in the receipt and
/// by the linter. See the README security boundary section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetMode {
    /// The command declares it needs no network.
    None,
    /// The command declares it may use the network (advisory only).
    Allow,
}

/// A fully validated policy ready to drive execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    /// Human-readable policy name.
    pub name: String,
    /// The program to run. May be a bare name (resolved via PATH) or a path.
    pub program: String,
    /// Whether the resolved program must live under one of `path.roots`.
    pub path_roots: Vec<String>,
    /// Fixed leading arguments always passed to the program.
    pub fixed_args: Vec<String>,
    /// Allow-list of additional arguments the caller may pass. If empty and
    /// `allow_extra_args` is false, no caller args are permitted.
    pub allowed_args: Vec<String>,
    /// When true, any caller-supplied argument is permitted (the arg
    /// allow-list is not consulted).
    pub allow_extra_args: bool,
    /// Environment handling mode.
    pub env_mode: EnvMode,
    /// Names to pass through from the parent environment (Clean mode).
    pub env_pass: Vec<String>,
    /// Names to remove from the inherited environment (Inherit mode).
    pub env_remove: Vec<String>,
    /// Explicit KEY=VALUE assignments applied last in both modes.
    pub env_set: Vec<(String, String)>,
    /// Working directory for the child, if specified.
    pub workdir: Option<String>,
    /// Declared network intent (advisory).
    pub net: NetMode,
    /// Wall-clock timeout in milliseconds. 0 means "no timeout".
    pub timeout_ms: u64,
}

/// A validation error, distinct from a syntactic [`ParseError`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "policy validation error: {}", self.message)
    }
}

impl std::error::Error for ValidationError {}
