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
// Exit codes:
//
//	0  success (no errors; also no warnings when --strict)
//	1  usage error
//	2  one or more policies had errors (or warnings under --strict)
//	3  a policy file could not be read or parsed
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

