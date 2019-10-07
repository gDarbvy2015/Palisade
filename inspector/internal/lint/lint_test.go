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
	}
}

func TestValidMinimalPolicyHasNoErrors(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = 1000\n")
	if r.HasErrors() {
		t.Errorf("expected no errors, got %+v", r.Diagnostics)
	}
}

func TestBadTimeoutIsError(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = abc\n")
	if !hasCode(r, "P002") {
		t.Errorf("expected P002, got %+v", r.Diagnostics)
	}
}

func TestNoTimeoutWarns(t *testing.T) {
