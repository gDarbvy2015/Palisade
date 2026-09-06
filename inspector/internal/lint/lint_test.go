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
		t.Errorf("expected P103, got %+v", r.Diagnostics)
	}
}

func TestUnknownKeyWarns(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = 1\nprogramm = typo\n")
	if !hasCode(r, "P105") {
		t.Errorf("expected P105 for unknown key, got %+v", r.Diagnostics)
	}
}

func TestUnknownSectionWarns(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = 1\n[exce]\nfoo = bar\n")
	if !hasCode(r, "P104") {
		t.Errorf("expected P104 for unknown section, got %+v", r.Diagnostics)
	}
}

func TestPathProgramWithoutRootIsInfo(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = /usr/bin/echo\ntimeout_ms = 1\n")
	if !hasCode(r, "P201") {
		t.Errorf("expected P201, got %+v", r.Diagnostics)
	}
}

func TestSeverityMarshalsAsString(t *testing.T) {
	d := Diagnostic{Severity: Warning, Code: "P100", Message: "m"}
	b, err := json.Marshal(d)
	if err != nil {
		t.Fatalf("marshal error: %v", err)
	}
	if !strings.Contains(string(b), `"severity":"warning"`) {
		t.Errorf("severity not marshaled as string: %s", b)
	}
}

func TestParseErrorPropagates(t *testing.T) {
	_, err := Lint("[unterminated\n")
	if err == nil {
		t.Errorf("expected parse error to propagate")
	}
}

func TestDiagnosticsAreOrderedByLine(t *testing.T) {
	r, _ := Lint("[exec]\nprogram = echo\ntimeout_ms = abc\nunknownkey = 1\n")
