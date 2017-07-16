//! Policy format parser.
//!
//! The palisade policy format is a small, line-oriented, INI-like format that is
//! intentionally trivial to parse without external dependencies and shares an
//! identical grammar with the Go `palisade-lint` tool.
//!
//! # Grammar (informal)
//!
//! ```text
//! # comments start with '#' and run to end of line
//! key = value                # scalar assignment
//! key += value               # append to a list-valued key
//! [section]                  # section header (currently: [exec], [env], [args], [path], [net])
//! ```
//!
//! Blank lines are ignored. Leading and trailing whitespace around keys and
//! values is trimmed. Values are always plain strings; there is no quoting or
//! escaping layer, which keeps the parser predictable and auditable. A value
//! that must contain a leading/trailing space is not supported by design.
//!
//! See `docs/POLICY.md` for the full specification.

use std::collections::BTreeMap;
use std::fmt;

/// A parse error with 1-based line context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "policy parse error on line {}: {}",
            self.line, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// A single parsed assignment, preserving source order and section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub section: String,
    pub key: String,
    pub value: String,
    pub append: bool,
