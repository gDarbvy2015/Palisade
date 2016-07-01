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
	r, err := Lint("name = x\n")
	if err != nil {
		t.Fatalf("parse error: %v", err)
	}
	if !hasCode(r, "P001") {
		t.Errorf("expected P001 for missing program, got %+v", r.Diagnostics)
	}
	if !r.HasErrors() {
		t.Errorf("expected HasErrors to be true")
