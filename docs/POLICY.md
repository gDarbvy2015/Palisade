# Palisade Policy Format

This document is the normative specification of the palisade policy format. Both
the Rust launcher (`palisade`) and the Go inspector (`palisade-lint`) implement
this grammar identically.

## 1. Lexical structure

A policy file is UTF-8 text interpreted line by line (`\n`; a trailing `\r` is
trimmed as surrounding whitespace).

- **Comments** begin with `#` and run to the end of the line. Because the format
  has no quoting layer, a `#` **anywhere** on a line begins a comment. To use a
