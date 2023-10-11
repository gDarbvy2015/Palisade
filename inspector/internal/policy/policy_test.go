package policy

import "testing"

func TestParsesScalarsAndSections(t *testing.T) {
	doc, err := Parse("name = demo\n[exec]\nprogram = echo\ntimeout_ms = 1000\n")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if v, _ := doc.Scalar("", "name"); v != "demo" {
		t.Errorf("name = %q, want demo", v)
	}
	if v, _ := doc.Scalar("exec", "program"); v != "echo" {
		t.Errorf("program = %q, want echo", v)
	}
	if v, _ := doc.Scalar("exec", "timeout_ms"); v != "1000" {
		t.Errorf("timeout_ms = %q, want 1000", v)
	}
}

func TestAccumulatesLists(t *testing.T) {
	doc, err := Parse("[args]\nallow = --version\nallow += --help\n")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := doc.List("args", "allow")
	want := []string{"--version", "--help"}
	if len(got) != len(want) {
		t.Fatalf("list len %d, want %d (%v)", len(got), len(want), got)
	}
	for i := range want {
		if got[i] != want[i] {
			t.Errorf("list[%d] = %q, want %q", i, got[i], want[i])
		}
	}
}

func TestLastScalarWins(t *testing.T) {
	doc, _ := Parse("k = a\nk = b\n")
	if v, _ := doc.Scalar("", "k"); v != "b" {
		t.Errorf("scalar = %q, want b", v)
	}
}

func TestStripsCommentsAndBlanks(t *testing.T) {
	doc, _ := Parse("# header\n\n  k = v  # trailing\n")
	if v, _ := doc.Scalar("", "k"); v != "v" {
		t.Errorf("scalar = %q, want v", v)
	}
}

func TestEmptyValueAllowed(t *testing.T) {
	doc, _ := Parse("k =\n")
	v, ok := doc.Scalar("", "k")
	if !ok || v != "" {
		t.Errorf("scalar = %q ok=%v, want empty ok", v, ok)
	}
}

func TestRejectsUnterminatedSection(t *testing.T) {
	_, err := Parse("[exec\n")
	pe, ok := err.(*ParseError)
	if !ok || pe.Line != 1 {
		t.Errorf("expected ParseError on line 1, got %v", err)
	}
}

func TestRejectsMissingEquals(t *testing.T) {
	_, err := Parse("garbage line\n")
	if _, ok := err.(*ParseError); !ok {
		t.Errorf("expected ParseError, got %v", err)
	}
}

func TestRejectsInvalidKey(t *testing.T) {
