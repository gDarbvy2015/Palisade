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
