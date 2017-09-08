# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-31

### Added

- **Policy format** (`docs/POLICY.md`): a tiny INI-like grammar with `=` scalars
  and `+=` list append, shared verbatim by both tools.
- **Rust launcher (`palisade`)**, stdlib only:
  - `policy` parser with line-accurate error reporting.
