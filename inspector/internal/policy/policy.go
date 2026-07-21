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

	lines := strings.Split(input, "\n")
	for idx, raw := range lines {
		lineNo := idx + 1
		line := strings.TrimSpace(stripComment(raw))
		if line == "" {
			continue
		}

		if strings.HasPrefix(line, "[") {
			if !strings.HasSuffix(line, "]") {
				return nil, &ParseError{Line: lineNo, Message: "unterminated section header (missing ']')"}
			}
			name := strings.TrimSpace(line[1 : len(line)-1])
			if name == "" {
				return nil, &ParseError{Line: lineNo, Message: "empty section name"}
			}
			if !isIdent(name) {
				return nil, &ParseError{Line: lineNo, Message: fmt.Sprintf("invalid section name %q", name)}
			}
			section = name
			continue
		}

		var key, value string
		var appendOp bool
		if pos := strings.Index(line, "+="); pos >= 0 {
			key = strings.TrimSpace(line[:pos])
			value = strings.TrimSpace(line[pos+2:])
			appendOp = true
		} else if pos := strings.IndexByte(line, '='); pos >= 0 {
			key = strings.TrimSpace(line[:pos])
			value = strings.TrimSpace(line[pos+1:])
		} else {
			return nil, &ParseError{Line: lineNo, Message: fmt.Sprintf("expected 'key = value' but found %q", line)}
		}

		if key == "" {
			return nil, &ParseError{Line: lineNo, Message: "empty key"}
		}
		if !isIdent(key) {
			return nil, &ParseError{Line: lineNo, Message: fmt.Sprintf("invalid key %q", key)}
		}

		doc.Assignments = append(doc.Assignments, Assignment{
			Section: section,
			Key:     key,
			Value:   value,
			Append:  appendOp,
			Line:    lineNo,
		})
	}

	return doc, nil
}

func stripComment(line string) string {
	if pos := strings.IndexByte(line, '#'); pos >= 0 {
		return line[:pos]
	}
	return line
}

func isIdent(s string) bool {
	if s == "" {
		return false
	}
	for _, c := range s {
		if !(c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c >= '0' && c <= '9' ||
			c == '_' || c == '-' || c == '.') {
			return false
		}
	}
	return true
}
