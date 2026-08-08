//! End-to-end tests that actually execute a child process through the library.
//!
//! These use platform-appropriate system programs so the suite runs on both
//! Windows and Unix without external fixtures.

use palisade::{execute, resolve, ExecPlan, Outcome, Policy};

/// Build a policy that runs a portable "print a string" command.
fn echo_policy(extra_allow: bool) -> String {
    if cfg!(windows) {
        // Use cmd.exe /C echo. Fixed args carry the /C echo hello.
        format!(
            "name = echo-test\n\
             [exec]\n\
             program = cmd\n\
             timeout_ms = 5000\n\
             [args]\n\
             fixed = /C\n\
             fixed += echo\n\
             fixed += hello\n\
             allow_extra = {extra_allow}\n\
             [env]\n\
             mode = clean\n"
        )
    } else {
        format!(
            "name = echo-test\n\
             [exec]\n\
             program = sh\n\
             timeout_ms = 5000\n\
             [args]\n\
             fixed = -c\n\
             fixed += echo hello\n\
             allow_extra = {extra_allow}\n\
             [env]\n\
             mode = clean\n"
        )
    }
}

fn run(policy_text: &str, extra: &[String]) -> palisade::Receipt {
    let policy = Policy::load(policy_text).expect("policy should load");
    policy.check_args(extra).expect("args should validate");
    let resolved = resolve(&policy.program, &policy.path_roots).expect("program resolves");
    let mut args = policy.fixed_args.clone();
    args.extend(extra.iter().cloned());
    let plan = ExecPlan {
        policy: &policy,
        resolved_path: resolved.display().to_string(),
        args,
    };
    execute(&plan).expect("execution succeeds")
}

#[test]
fn executes_and_exits_zero() {
    let receipt = run(&echo_policy(false), &[]);
    assert_eq!(receipt.outcome, Outcome::Exited(Some(0)));
    assert!(receipt.duration_ms < 5000);
    assert_eq!(receipt.net_declared, "none");
}

#[test]
fn timeout_kills_long_child() {
    let policy_text = if cfg!(windows) {
        // ping -n 5 sleeps ~4s; timeout at 300ms forces a kill.
        "name = slow\n\
         [exec]\n\
         program = cmd\n\
         timeout_ms = 300\n\
         [args]\n\
         fixed = /C\n\
         fixed += ping -n 5 127.0.0.1 > NUL\n\
         [env]\n\
         mode = inherit\n"
            .to_string()
    } else {
        "name = slow\n\
         [exec]\n\
         program = sh\n\
         timeout_ms = 300\n\
         [args]\n\
         fixed = -c\n\
         fixed += sleep 5\n\
         [env]\n\
         mode = clean\n"
            .to_string()
    };
    let receipt = run(&policy_text, &[]);
    assert_eq!(receipt.outcome, Outcome::TimedOut);
    // Should terminate close to the timeout, not run the full 5 seconds.
    assert!(
        receipt.duration_ms < 3000,
        "expected quick timeout, got {} ms",
        receipt.duration_ms
    );
}

#[test]
fn receipt_json_roundtrips_keys() {
    let receipt = run(&echo_policy(false), &[]);
    let json = receipt.to_json();
    for key in [
        "policy_name",
        "program",
        "resolved_path",
        "args",
        "env_mode",
        "effective_env_keys",
        "net_declared",
        "timeout_ms",
        "duration_ms",
        "outcome",
        "exit_code",
    ] {
        assert!(
            json.contains(&format!("\"{key}\"")),
            "missing key {key} in {json}"
