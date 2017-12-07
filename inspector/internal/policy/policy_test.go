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
