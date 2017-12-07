//! Process execution with environment filtering and a wall-clock timeout.
//!
//! Timeout implementation is portable and stdlib-only: we spawn the child, then
//! poll `try_wait` on a short interval until either the child exits or the
//! deadline passes. On timeout we send a best-effort kill. This is cooperative
//! and portable; it does not rely on platform-specific timers or signals.

use crate::model::{EnvMode, Policy};
use std::collections::BTreeMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The outcome of an execution attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Child exited on its own with the given code (None if terminated by a
    /// signal on Unix and no code is available).
    Exited(Option<i32>),
    /// The child exceeded the timeout and was killed.
    TimedOut,
}

/// A record of what was executed and how it ended — the "receipt".
#[derive(Debug, Clone)]
pub struct Receipt {
    pub policy_name: String,
    pub program: String,
    pub resolved_path: String,
    pub args: Vec<String>,
    pub env_mode: String,
    pub effective_env_keys: Vec<String>,
    pub net_declared: String,
    pub timeout_ms: u64,
    pub duration_ms: u128,
    pub outcome: Outcome,
}

impl Receipt {
    /// Serialize the receipt to a compact, dependency-free JSON string.
    ///
    /// We hand-roll JSON to keep the crate stdlib-only. All values are simple
    /// strings, numbers, arrays of strings, and booleans, so escaping is
    /// limited to the small set handled by [`json_escape`].
    pub fn to_json(&self) -> String {
        let (outcome_kind, code) = match &self.outcome {
            Outcome::Exited(Some(c)) => ("exited", Some(*c)),
