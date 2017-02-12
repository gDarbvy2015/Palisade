//! `palisade` command-line interface.
//!
//! Usage:
//!   palisade run   --policy <FILE> [--receipt <FILE>] [-- <extra args>...]
//!   palisade check --policy <FILE>
//!   palisade help
//!   palisade version
//!
//! Exit codes:
//!   0   child exited 0 / check succeeded
//!   1   usage error
//!   2   policy load/validation error
//!   3   executable resolution error
//!   4   argument validation error
//!   5   spawn/wait error
//!   124 child timed out (matches GNU `timeout` convention)
//!   N   otherwise, the child's own non-zero exit code

use palisade::{execute, resolve, ExecPlan, Outcome, Policy};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "\
palisade — userspace declarative command policy launcher

USAGE:
    palisade run   --policy <FILE> [--receipt <FILE>] [-- <extra args>...]
    palisade check --policy <FILE>
    palisade help
    palisade version

COMMANDS:
