//! Palisade launcher library.
//!
//! Palisade is a **userspace declarative command policy launcher**. It reads a
//! small policy file describing exactly one program, the arguments and
//! environment it is permitted to run with, an optional path-root constraint,
//! a declared (advisory) network intent, and a wall-clock timeout. It then
