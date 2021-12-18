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
