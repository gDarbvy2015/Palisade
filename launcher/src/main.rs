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
        }
    }
}

struct CliError {
    message: String,
    code: ExitCode,
}

fn err(message: impl Into<String>, code: u8) -> CliError {
    CliError {
        message: message.into(),
        code: ExitCode::from(code),
    }
}

fn run(args: &[String]) -> Result<ExitCode, CliError> {
    let Some(command) = args.first() else {
        eprintln!("{USAGE}");
        return Err(err("no command given", 1));
    };

    match command.as_str() {
        "help" | "-h" | "--help" => {
            println!("{USAGE}");
            Ok(ExitCode::SUCCESS)
        }
        "version" | "-V" | "--version" => {
            println!("palisade {VERSION}");
            Ok(ExitCode::SUCCESS)
        }
        "check" => cmd_check(&args[1..]),
        "run" => cmd_run(&args[1..]),
        other => {
            eprintln!("{USAGE}");
            Err(err(format!("unknown command '{other}'"), 1))
        }
    }
}

/// Parsed options common to run/check.
struct Opts {
    policy_path: String,
    receipt_path: Option<String>,
    extra: Vec<String>,
}

fn parse_opts(args: &[String]) -> Result<Opts, CliError> {
    let mut policy_path: Option<String> = None;
    let mut receipt_path: Option<String> = None;
    let mut extra: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--policy" => {
                i += 1;
                policy_path = Some(
                    args.get(i)
                        .ok_or_else(|| err("--policy requires a value", 1))?
                        .clone(),
                );
            }
            "--receipt" => {
                i += 1;
                receipt_path = Some(
                    args.get(i)
                        .ok_or_else(|| err("--receipt requires a value", 1))?
                        .clone(),
                );
            }
            "--" => {
                extra.extend_from_slice(&args[i + 1..]);
                break;
            }
            other => {
                return Err(err(format!("unexpected argument '{other}'"), 1));
            }
        }
        i += 1;
    }

    let policy_path = policy_path.ok_or_else(|| err("--policy <FILE> is required", 1))?;
    Ok(Opts {
        policy_path,
        receipt_path,
        extra,
    })
}

fn load_policy(path: &str) -> Result<Policy, CliError> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| err(format!("cannot read policy '{path}': {e}"), 2))?;
    Policy::load(&text).map_err(|e| err(e.to_string(), 2))
}

fn cmd_check(args: &[String]) -> Result<ExitCode, CliError> {
    let opts = parse_opts(args)?;
    let policy = load_policy(&opts.policy_path)?;
    // Resolve as part of the check so users learn early if the program is
    // missing, but do not execute.
    let resolved =
        resolve(&policy.program, &policy.path_roots).map_err(|e| err(e.to_string(), 3))?;
    println!("policy '{}' is valid", policy.name);
    println!("  program:  {} -> {}", policy.program, resolved.display());
    println!("  timeout:  {} ms", policy.timeout_ms);
    println!(
        "  net:      {}",
        match policy.net {
            palisade::NetMode::None => "none (declared)",
            palisade::NetMode::Allow => "allow (declared, advisory only)",
        }
    );
    println!("  fixed args: {:?}", policy.fixed_args);
    if policy.allow_extra_args {
        println!("  extra args: any (allow_extra = true)");
    } else {
        println!("  extra args: allow-list {:?}", policy.allowed_args);
    }
    Ok(ExitCode::SUCCESS)
}

