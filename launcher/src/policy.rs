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
