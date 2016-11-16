//! End-to-end tests that actually execute a child process through the library.
//!
//! These use platform-appropriate system programs so the suite runs on both
//! Windows and Unix without external fixtures.

use palisade::{execute, resolve, ExecPlan, Outcome, Policy};

/// Build a policy that runs a portable "print a string" command.
fn echo_policy(extra_allow: bool) -> String {
