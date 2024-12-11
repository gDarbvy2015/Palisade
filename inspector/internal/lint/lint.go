// Package lint validates palisade policies at the semantic level and produces
// structured diagnostics. It mirrors the validation rules enforced by the Rust
// launcher's model layer, plus additional advisory checks (best-practice hints)
// that help authors write safer, more explicit policies.
package lint

import (
	"fmt"
	"sort"
	"strconv"
	"strings"

	"github.com/example/palisade/inspector/internal/policy"
)

// Severity classifies a diagnostic.
type Severity int

const (
	// Error means the policy would be rejected by the launcher.
	Error Severity = iota
	// Warning means the policy is valid but likely unsafe or surprising.
	Warning
	// Info is an advisory note.
	Info
)

func (s Severity) String() string {
	switch s {
	case Error:
		return "error"
	case Warning:
		return "warning"
	case Info:
		return "info"
	default:
		return "unknown"
	}
}

// MarshalJSON emits the severity as its string form for readable JSON.
func (s Severity) MarshalJSON() ([]byte, error) {
	return []byte(`"` + s.String() + `"`), nil
}

// Diagnostic is a single finding.
type Diagnostic struct {
	Severity Severity `json:"severity"`
	Code     string   `json:"code"`
	Message  string   `json:"message"`
	Line     int      `json:"line,omitempty"`
}

// Report is the full set of diagnostics for a policy.
type Report struct {
	Diagnostics []Diagnostic `json:"diagnostics"`
}

// HasErrors reports whether any diagnostic is an error.
func (r *Report) HasErrors() bool {
	for _, d := range r.Diagnostics {
		if d.Severity == Error {
			return true
		}
	}
	return false
}

// Counts returns the number of errors, warnings, and infos.
func (r *Report) Counts() (errors, warnings, infos int) {
	for _, d := range r.Diagnostics {
		switch d.Severity {
		case Error:
			errors++
		case Warning:
			warnings++
		case Info:
			infos++
		}
	}
	return
}

// knownSections and their known keys. Unknown sections/keys generate warnings,
// which catches typos like [exce] or "programm".
var knownKeys = map[string]map[string]bool{
	"": {
		"name": true,
	},
	"exec": {
		"program":    true,
		"timeout_ms": true,
		"workdir":    true,
	},
	"args": {
		"fixed":       true,
		"allow":       true,
		"allow_extra": true,
	},
	"env": {
		"mode":   true,
		"pass":   true,
		"remove": true,
		"set":    true,
	},
	"path": {
		"root": true,
	},
	"net": {
		"mode": true,
	},
}

// Lint runs all checks against parsed policy text and returns a Report.
func Lint(input string) (*Report, error) {
	doc, err := policy.Parse(input)
	if err != nil {
		return nil, err
	}
	r := &Report{}

	r.checkRequired(doc)
	r.checkTimeout(doc)
	r.checkEnvMode(doc)
	r.checkEnvSet(doc)
	r.checkNetMode(doc)
	r.checkArgsExtra(doc)
	r.checkUnknownKeys(doc)
	r.checkAdvisories(doc)

	// Deterministic ordering: by line, then severity, then code.
	sort.SliceStable(r.Diagnostics, func(i, j int) bool {
		a, b := r.Diagnostics[i], r.Diagnostics[j]
		if a.Line != b.Line {
			return a.Line < b.Line
		}
		if a.Severity != b.Severity {
			return a.Severity < b.Severity
		}
		return a.Code < b.Code
	})
	return r, nil
}

func (r *Report) add(sev Severity, code, msg string, line int) {
	r.Diagnostics = append(r.Diagnostics, Diagnostic{Severity: sev, Code: code, Message: msg, Line: line})
}

func (r *Report) checkRequired(doc *policy.Document) {
	if v, ok := doc.Scalar("exec", "program"); !ok || strings.TrimSpace(v) == "" {
		r.add(Error, "P001", "missing required key 'exec.program'", 0)
	}
}

func (r *Report) checkTimeout(doc *policy.Document) {
	v, ok := doc.Scalar("exec", "timeout_ms")
	if !ok {
		r.add(Warning, "P100", "no 'exec.timeout_ms' set; the child may run without a time limit", 0)
		return
	}
	n, err := strconv.ParseUint(strings.TrimSpace(v), 10, 64)
	if err != nil {
		r.add(Error, "P002", fmt.Sprintf("'exec.timeout_ms' must be a non-negative integer, got %q", v), lineOf(doc, "exec", "timeout_ms"))
		return
	}
	if n == 0 {
		r.add(Warning, "P101", "'exec.timeout_ms = 0' disables the timeout entirely", lineOf(doc, "exec", "timeout_ms"))
	}
}

func (r *Report) checkEnvMode(doc *policy.Document) {
	v, ok := doc.Scalar("env", "mode")
	if !ok {
		return // defaults to clean
	}
	switch v {
	case "clean", "inherit":
	default:
		r.add(Error, "P003", fmt.Sprintf("'env.mode' must be 'clean' or 'inherit', got %q", v), lineOf(doc, "env", "mode"))
	}
	if v == "inherit" {
		r.add(Warning, "P102", "'env.mode = inherit' exposes the full parent environment to the child; prefer 'clean' with an explicit pass list", lineOf(doc, "env", "mode"))
	}
}

func (r *Report) checkEnvSet(doc *policy.Document) {
	for _, entry := range doc.List("env", "set") {
		if !strings.Contains(entry, "=") || strings.HasPrefix(strings.TrimSpace(entry), "=") {
			r.add(Error, "P004", fmt.Sprintf("'env.set' entries must be KEY=VALUE, got %q", entry), lineOf(doc, "env", "set"))
		}
	}
}

func (r *Report) checkNetMode(doc *policy.Document) {
	v, ok := doc.Scalar("net", "mode")
	if !ok {
		return
	}
	switch v {
	case "none", "allow":
	default:
		r.add(Error, "P005", fmt.Sprintf("'net.mode' must be 'none' or 'allow', got %q", v), lineOf(doc, "net", "mode"))
	}
	if v == "allow" {
		r.add(Info, "P200", "'net.mode = allow' is advisory only; palisade cannot enforce network access in userspace", lineOf(doc, "net", "mode"))
	}
}

func (r *Report) checkArgsExtra(doc *policy.Document) {
	v, ok := doc.Scalar("args", "allow_extra")
	if !ok {
		return
	}
	switch strings.ToLower(v) {
	case "true", "yes", "1", "on":
		r.add(Warning, "P103", "'args.allow_extra = true' lets callers pass arbitrary arguments; the args allow-list is ignored", lineOf(doc, "args", "allow_extra"))
	case "false", "no", "0", "off":
	default:
