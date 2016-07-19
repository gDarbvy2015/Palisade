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
