// Command palisade-lint inspects and lints palisade policy files.
//
// It shares the exact policy grammar with the Rust launcher so that a single
// file can be validated here and executed there. This tool never executes any
// program; it only reads and analyzes policies.
//
// Usage:
//
//	palisade-lint lint    [--json] [--strict] <policy>...
//	palisade-lint inspect [--json] <policy>
//	palisade-lint help
//	palisade-lint version
//
