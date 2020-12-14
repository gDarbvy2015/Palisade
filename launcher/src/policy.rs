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
    pub line: usize,
}

/// The raw parsed document: an ordered list of assignments plus a fast lookup.
///
/// Scalar keys resolve to the last assignment; list keys accumulate via `+=`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub assignments: Vec<Assignment>,
}

impl Document {
    /// Fully-qualified key = `section.key` (or just `key` when section is empty).
    fn fq(section: &str, key: &str) -> String {
        if section.is_empty() {
            key.to_string()
        } else {
            format!("{section}.{key}")
        }
    }

    /// Return the last scalar value for `section.key`, ignoring appends order
    /// beyond "last wins".
    pub fn scalar(&self, section: &str, key: &str) -> Option<&str> {
        self.assignments
            .iter()
            .rev()
            .find(|a| a.section == section && a.key == key && !a.append)
            .map(|a| a.value.as_str())
    }

    /// Return all values for a list key (both the initial `=` and any `+=`),
    /// in source order.
    pub fn list(&self, section: &str, key: &str) -> Vec<&str> {
        self.assignments
            .iter()
            .filter(|a| a.section == section && a.key == key)
            .map(|a| a.value.as_str())
            .collect()
    }

    /// A stable, sorted map of fully-qualified keys to their joined values,
    /// primarily for diagnostics and receipts.
    pub fn summary(&self) -> BTreeMap<String, Vec<String>> {
        let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for a in &self.assignments {
            map.entry(Self::fq(&a.section, &a.key))
                .or_default()
                .push(a.value.clone());
        }
        map
    }
}

/// Parse the policy text into a [`Document`].
pub fn parse(input: &str) -> Result<Document, ParseError> {
    let mut doc = Document::default();
    let mut section = String::new();

    for (idx, raw) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix('[') {
            let name = rest.strip_suffix(']').ok_or_else(|| ParseError {
                line: line_no,
                message: "unterminated section header (missing ']')".to_string(),
            })?;
            let name = name.trim();
            if name.is_empty() {
                return Err(ParseError {
                    line: line_no,
                    message: "empty section name".to_string(),
                });
            }
            if !is_ident(name) {
                return Err(ParseError {
                    line: line_no,
                    message: format!("invalid section name '{name}'"),
                });
