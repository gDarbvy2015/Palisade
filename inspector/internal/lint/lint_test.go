package lint

import (
	"encoding/json"
	"strings"
	"testing"
)

func hasCode(r *Report, code string) bool {
	for _, d := range r.Diagnostics {
		if d.Code == code {
			return true
		}
	}
	return false
}

func TestMissingProgramIsError(t *testing.T) {
