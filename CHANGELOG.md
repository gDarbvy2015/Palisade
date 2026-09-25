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
  - `model` typed policy with validation (required program, timeout, env mode,
    env.set KEY=VALUE, net mode, argument allow-list).
  - `resolve` cross-platform executable resolution (PATH + `PATHEXT` on Windows)
    with optional `path.root` confinement.
  - `exec` environment filtering (`clean` / `inherit`), portable wall-clock
    timeout via poll-and-kill, and a hand-rolled JSON **receipt**.
  - `run`, `check`, `help`, and `version` subcommands with documented exit codes
    (including `124` for timeouts).
- **Go inspector (`palisade-lint`)**, stdlib only:
  - `policy` parser mirroring the Rust grammar.
  - `lint` semantic checks producing coded diagnostics (errors `P0xx`, warnings
    `P1xx`, info `P2xx`) with deterministic ordering.
  - `lint` and `inspect` subcommands with `--json` and `--strict` options.
- **Examples**: `echo`, `git-status`, and `network-fetch` policies.
- **Tests**: 29 Rust tests (unit + real process-execution integration incl. a
  timeout kill) and 22 Go tests (parser + linter).
- Project scaffolding: README with Mermaid diagram and an honest security
  boundary, `Makefile`, GitHub Actions CI for both languages, `.gitignore`,
  `.editorconfig`, MIT `LICENSE`.

[0.1.0]: https://github.com/example/palisade/releases/tag/v0.1.0
