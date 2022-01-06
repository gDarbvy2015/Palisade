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
	}
	return "", false
}

// List returns all values for section.key in source order.
func (d *Document) List(section, key string) []string {
	var out []string
	for _, a := range d.Assignments {
		if a.Section == section && a.Key == key {
			out = append(out, a.Value)
		}
	}
	return out
}

// Keys returns the sorted set of fully-qualified keys present in the document.
func (d *Document) Keys() []string {
	seen := map[string]struct{}{}
	for _, a := range d.Assignments {
		seen[fq(a.Section, a.Key)] = struct{}{}
	}
	out := make([]string, 0, len(seen))
	for k := range seen {
		out = append(out, k)
	}
	sort.Strings(out)
	return out
}

func fq(section, key string) string {
	if section == "" {
		return key
	}
	return section + "." + key
}

// Parse converts policy text into a Document, or returns a *ParseError.
func Parse(input string) (*Document, error) {
	doc := &Document{}
	section := ""

