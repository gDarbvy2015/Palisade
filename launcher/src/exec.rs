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
        if i > 0 {
            s.push(',');
        }
        s.push('"');
        s.push_str(&json_escape(v));
        s.push('"');
    }
    s.push(']');
}

/// Escape a string for embedding in JSON.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Compute the effective environment for the child according to the policy.
///
/// Returns a sorted map so the resulting environment (and the receipt) is
/// deterministic.
pub fn effective_env(policy: &Policy) -> BTreeMap<String, String> {
    let mut env: BTreeMap<String, String> = BTreeMap::new();

    match policy.env_mode {
        EnvMode::Clean => {
            for name in &policy.env_pass {
                if let Some(val) = std::env::var_os(name) {
                    env.insert(name.clone(), val.to_string_lossy().into_owned());
                }
            }
        }
        EnvMode::Inherit => {
            for (k, v) in std::env::vars() {
                env.insert(k, v);
            }
            for name in &policy.env_remove {
                env.remove(name);
            }
        }
    }

    for (k, v) in &policy.env_set {
        env.insert(k.clone(), v.clone());
    }

    env
}

/// Configuration for a single execution.
pub struct ExecPlan<'a> {
    pub policy: &'a Policy,
    pub resolved_path: String,
    pub args: Vec<String>,
}

/// Errors that can arise while launching the child.
#[derive(Debug)]
pub enum ExecError {
    Spawn(String),
    Wait(String),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::Spawn(m) => write!(f, "failed to spawn child process: {m}"),
            ExecError::Wait(m) => write!(f, "failed while waiting on child process: {m}"),
        }
    }
}

impl std::error::Error for ExecError {}

/// Execute the plan, applying the environment and timeout, and produce a
/// [`Receipt`].
pub fn execute(plan: &ExecPlan) -> Result<Receipt, ExecError> {
    let policy = plan.policy;
    let env = effective_env(policy);

    let mut cmd = Command::new(&plan.resolved_path);
    cmd.args(&plan.args);
    cmd.env_clear();
    for (k, v) in &env {
        cmd.env(k, v);
    }
    if let Some(dir) = &policy.workdir {
        cmd.current_dir(dir);
    }
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    let start = Instant::now();
    let mut child = cmd.spawn().map_err(|e| ExecError::Spawn(e.to_string()))?;

    let outcome = if policy.timeout_ms == 0 {
        let status = child.wait().map_err(|e| ExecError::Wait(e.to_string()))?;
        Outcome::Exited(status.code())
    } else {
        let deadline = start + Duration::from_millis(policy.timeout_ms);
        let poll = Duration::from_millis(pick_poll_interval(policy.timeout_ms));
        loop {
            match child
                .try_wait()
                .map_err(|e| ExecError::Wait(e.to_string()))?
            {
                Some(status) => break Outcome::Exited(status.code()),
                None => {
                    if Instant::now() >= deadline {
                        // Best-effort termination. On all platforms `kill`
                        // requests termination; we then reap the child.
                        let _ = child.kill();
                        let _ = child.wait();
                        break Outcome::TimedOut;
                    }
                    std::thread::sleep(poll);
                }
            }
        }
    };

    let duration_ms = start.elapsed().as_millis();

    Ok(Receipt {
        policy_name: policy.name.clone(),
        program: policy.program.clone(),
        resolved_path: plan.resolved_path.clone(),
        args: plan.args.clone(),
        env_mode: match policy.env_mode {
            EnvMode::Clean => "clean".to_string(),
            EnvMode::Inherit => "inherit".to_string(),
        },
        effective_env_keys: env.keys().cloned().collect(),
        net_declared: match policy.net {
            crate::model::NetMode::None => "none".to_string(),
            crate::model::NetMode::Allow => "allow".to_string(),
        },
        timeout_ms: policy.timeout_ms,
        duration_ms,
        outcome,
    })
}

