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
            Outcome::Exited(None) => ("exited", None),
            Outcome::TimedOut => ("timed_out", None),
        };

        let mut s = String::new();
        s.push('{');
        push_kv_str(&mut s, "policy_name", &self.policy_name, true);
        push_kv_str(&mut s, "program", &self.program, false);
        push_kv_str(&mut s, "resolved_path", &self.resolved_path, false);
        push_kv_arr(&mut s, "args", &self.args, false);
        push_kv_str(&mut s, "env_mode", &self.env_mode, false);
        push_kv_arr(
            &mut s,
            "effective_env_keys",
            &self.effective_env_keys,
            false,
        );
        push_kv_str(&mut s, "net_declared", &self.net_declared, false);
        push_kv_num(&mut s, "timeout_ms", self.timeout_ms as i128, false);
        push_kv_num(&mut s, "duration_ms", self.duration_ms as i128, false);
        push_kv_str(&mut s, "outcome", outcome_kind, false);
        match code {
            Some(c) => push_kv_num(&mut s, "exit_code", c as i128, false),
            None => {
                s.push(',');
                s.push_str("\"exit_code\":null");
            }
        }
        s.push('}');
        s
    }
}

fn push_comma(s: &mut String, first: bool) {
    if !first {
        s.push(',');
    }
}

fn push_kv_str(s: &mut String, key: &str, val: &str, first: bool) {
    push_comma(s, first);
    s.push('"');
    s.push_str(&json_escape(key));
    s.push_str("\":\"");
    s.push_str(&json_escape(val));
    s.push('"');
}

fn push_kv_num(s: &mut String, key: &str, val: i128, first: bool) {
    push_comma(s, first);
    s.push('"');
    s.push_str(&json_escape(key));
    s.push_str("\":");
    s.push_str(&val.to_string());
}

fn push_kv_arr(s: &mut String, key: &str, vals: &[String], first: bool) {
    push_comma(s, first);
    s.push('"');
    s.push_str(&json_escape(key));
    s.push_str("\":[");
    for (i, v) in vals.iter().enumerate() {
