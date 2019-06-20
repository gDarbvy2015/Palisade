// Package policy parses the palisade policy format. Its grammar is identical
// to the Rust launcher's parser (see launcher/src/policy.rs and docs/POLICY.md)
// so that a single policy file is understood the same way by both tools.
//
// The format is a small, line-oriented, INI-like language:
//
//	# comments start with '#'
//	key = value        # scalar assignment
//	key += value       # append to a list-valued key
//	[section]          # section header
//
// Values are plain strings; there is no quoting or escaping layer. Whitespace
// around keys and values is trimmed. A '#' anywhere begins a comment.
package policy

import (
	"fmt"
	"sort"
	"strings"
)

// ParseError carries a 1-based line number and a message.
type ParseError struct {
	Line    int
	Message string
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("policy parse error on line %d: %s", e.Line, e.Message)
}

// Assignment is a single parsed key/value, preserving section and source order.
type Assignment struct {
	Section string
	Key     string
	Value   string
	Append  bool
	Line    int
}

// Document is the ordered set of assignments parsed from a policy file.
type Document struct {
	Assignments []Assignment
}

// Scalar returns the last non-append value for section.key, if any.
func (d *Document) Scalar(section, key string) (string, bool) {
	for i := len(d.Assignments) - 1; i >= 0; i-- {
		a := d.Assignments[i]
		if a.Section == section && a.Key == key && !a.Append {
			return a.Value, true
		}
