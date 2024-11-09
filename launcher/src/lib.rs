//! Palisade launcher library.
//!
//! Palisade is a **userspace declarative command policy launcher**. It reads a
//! small policy file describing exactly one program, the arguments and
//! environment it is permitted to run with, an optional path-root constraint,
//! a declared (advisory) network intent, and a wall-clock timeout. It then
//! resolves the executable, filters the environment, validates arguments,
//! launches the process, enforces the timeout, and writes a machine-readable
//! receipt.
//!
//! ## What this is NOT
//!
//! Palisade does **not** provide OS kernel sandboxing. It does not use
//! seccomp, namespaces, cgroups, AppContainer, job objects, or any
//! kernel-enforced isolation. A launched program that is determined to break
//! out (spawning children, opening sockets despite `net.mode = none`, reading
//! files outside a root) can do so. Palisade reduces *accidental* exposure and
//! makes intent explicit and auditable; it is not a security boundary against
//! a hostile binary. See the README "Security boundary" section.

pub mod exec;
pub mod model;
pub mod policy;
pub mod resolve;

