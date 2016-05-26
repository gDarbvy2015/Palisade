package policy

import "testing"

func TestParsesScalarsAndSections(t *testing.T) {
	doc, err := Parse("name = demo\n[exec]\nprogram = echo\ntimeout_ms = 1000\n")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
