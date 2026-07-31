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

/// Umbrella error for loading a policy from text.
#[derive(Debug)]
pub enum LoadError {
    Parse(ParseError),
    Validate(ValidationError),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Parse(e) => write!(f, "{e}"),
            LoadError::Validate(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LoadError {}

impl From<ParseError> for LoadError {
    fn from(e: ParseError) -> Self {
        LoadError::Parse(e)
    }
}

impl From<ValidationError> for LoadError {
    fn from(e: ValidationError) -> Self {
        LoadError::Validate(e)
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Some(true),
        "false" | "no" | "0" | "off" => Some(false),
        _ => None,
    }
}

fn split_kv(pair: &str) -> Option<(String, String)> {
    let pos = pair.find('=')?;
    let (k, v) = pair.split_at(pos);
    let k = k.trim();
    if k.is_empty() {
        return None;
    }
    Some((k.to_string(), v[1..].trim().to_string()))
}

impl Policy {
    /// Parse and validate a policy from source text.
    pub fn load(input: &str) -> Result<Policy, LoadError> {
        let doc = parse(input)?;
        Ok(Policy::from_document(&doc)?)
    }

    /// Build a validated policy from an already-parsed document.
    pub fn from_document(doc: &Document) -> Result<Policy, ValidationError> {
        let err = |m: String| ValidationError { message: m };

        let name = doc.scalar("", "name").unwrap_or("unnamed").to_string();

        let program = doc
            .scalar("exec", "program")
            .ok_or_else(|| err("missing required key 'exec.program'".to_string()))?
            .to_string();
        if program.is_empty() {
            return Err(err("'exec.program' must not be empty".to_string()));
        }

        let timeout_ms = match doc.scalar("exec", "timeout_ms") {
            None => 0,
            Some(v) => v.parse::<u64>().map_err(|_| {
                err(format!(
                    "'exec.timeout_ms' must be a non-negative integer, got '{v}'"
                ))
            })?,
        };

        let workdir = doc.scalar("exec", "workdir").map(|s| s.to_string());

        let fixed_args = doc
            .list("args", "fixed")
            .iter()
            .map(|s| s.to_string())
            .collect();
        let allowed_args = doc
            .list("args", "allow")
            .iter()
            .map(|s| s.to_string())
            .collect();
        let allow_extra_args = match doc.scalar("args", "allow_extra") {
            None => false,
            Some(v) => parse_bool(v)
                .ok_or_else(|| err(format!("'args.allow_extra' must be a boolean, got '{v}'")))?,
        };

        let env_mode = match doc.scalar("env", "mode") {
            None | Some("clean") => EnvMode::Clean,
            Some("inherit") => EnvMode::Inherit,
            Some(other) => {
                return Err(err(format!(
                    "'env.mode' must be 'clean' or 'inherit', got '{other}'"
                )))
            }
        };
        let env_pass = doc
            .list("env", "pass")
            .iter()
            .map(|s| s.to_string())
            .collect();
        let env_remove = doc
            .list("env", "remove")
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut env_set = Vec::new();
        for raw in doc.list("env", "set") {
            let kv = split_kv(raw)
                .ok_or_else(|| err(format!("'env.set' entries must be KEY=VALUE, got '{raw}'")))?;
            env_set.push(kv);
        }

        let path_roots = doc
            .list("path", "root")
            .iter()
            .map(|s| s.to_string())
            .collect();

        let net = match doc.scalar("net", "mode") {
            None | Some("none") => NetMode::None,
            Some("allow") => NetMode::Allow,
            Some(other) => {
                return Err(err(format!(
                    "'net.mode' must be 'none' or 'allow', got '{other}'"
                )))
            }
        };

        Ok(Policy {
            name,
            program,
            path_roots,
            fixed_args,
            allowed_args,
            allow_extra_args,
            env_mode,
            env_pass,
            env_remove,
            env_set,
            workdir,
            net,
            timeout_ms,
        })
    }

    /// Validate that a set of caller-supplied arguments is permitted by the
    /// argument policy. Returns an error listing the first disallowed value.
    pub fn check_args(&self, extra: &[String]) -> Result<(), ValidationError> {
        if self.allow_extra_args {
            return Ok(());
        }
        for a in extra {
            if !self.allowed_args.iter().any(|allowed| allowed == a) {
                return Err(ValidationError {
                    message: format!(
                        "argument '{a}' is not in the allow-list (args.allow); \
                         set args.allow_extra = true to permit arbitrary arguments"
                    ),
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
name = echo-demo
[exec]
program = echo
timeout_ms = 2000
[args]
fixed = hello
allow = --version
allow += -n
[env]
mode = clean
pass = HOME
set = GREETING=hi
[net]
mode = none
";

    #[test]
    fn loads_full_policy() {
        let p = Policy::load(SAMPLE).unwrap();
        assert_eq!(p.name, "echo-demo");
        assert_eq!(p.program, "echo");
        assert_eq!(p.timeout_ms, 2000);
        assert_eq!(p.fixed_args, vec!["hello"]);
        assert_eq!(p.allowed_args, vec!["--version", "-n"]);
        assert_eq!(p.env_mode, EnvMode::Clean);
        assert_eq!(p.env_pass, vec!["HOME"]);
        assert_eq!(p.env_set, vec![("GREETING".to_string(), "hi".to_string())]);
        assert_eq!(p.net, NetMode::None);
    }

    #[test]
    fn missing_program_is_error() {
        let e = Policy::load("name = x\n").unwrap_err();
        assert!(matches!(e, LoadError::Validate(_)));
    }

    #[test]
    fn bad_timeout_is_error() {
        let e = Policy::load("[exec]\nprogram = echo\ntimeout_ms = abc\n").unwrap_err();
        assert!(matches!(e, LoadError::Validate(_)));
    }

    #[test]
    fn args_allow_list_enforced() {
        let p = Policy::load(SAMPLE).unwrap();
        assert!(p.check_args(&["--version".to_string()]).is_ok());
        assert!(p.check_args(&["--evil".to_string()]).is_err());
