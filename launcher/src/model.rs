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
