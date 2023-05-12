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
	r, _ := Lint("[exec]\nprogram = echo\n")
	if !hasCode(r, "P100") {
		t.Errorf("expected P100 warning, got %+v", r.Diagnostics)
	}
}

func TestInheritEnvWarns(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = 1\n[env]\nmode = inherit\n")
	if !hasCode(r, "P102") {
		t.Errorf("expected P102 warning, got %+v", r.Diagnostics)
	}
}

func TestBadEnvModeIsError(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = 1\n[env]\nmode = bogus\n")
	if !hasCode(r, "P003") {
		t.Errorf("expected P003, got %+v", r.Diagnostics)
	}
}

func TestBadEnvSetIsError(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = 1\n[env]\nset = NOEQUALS\n")
	if !hasCode(r, "P004") {
		t.Errorf("expected P004, got %+v", r.Diagnostics)
	}
}

func TestNetAllowIsInfo(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = curl\ntimeout_ms = 1\n[net]\nmode = allow\n")
	if !hasCode(r, "P200") {
		t.Errorf("expected P200 info, got %+v", r.Diagnostics)
	}
}

func TestAllowExtraWarns(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = 1\n[args]\nallow_extra = true\n")
	if !hasCode(r, "P103") {
