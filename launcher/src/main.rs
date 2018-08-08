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
    run      Load a policy, validate, and execute the described program.
    check    Load and validate a policy without executing anything.
    help     Show this help.
    version  Print the version.

OPTIONS:
    --policy <FILE>    Path to the policy file (required for run/check).
    --receipt <FILE>   Write the execution receipt (JSON) to FILE.
                       If omitted, the receipt is printed to stderr.
    --                 Everything after this is passed as extra arguments to
                       the child, subject to the args allow-list.

NOTE:
    This is a userspace launcher, not a kernel sandbox. See the README.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("palisade: {}", e.message);
            e.code
