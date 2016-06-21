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
