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
