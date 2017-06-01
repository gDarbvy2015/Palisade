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
