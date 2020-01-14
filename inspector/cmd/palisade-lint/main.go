// Command palisade-lint inspects and lints palisade policy files.
//
// It shares the exact policy grammar with the Rust launcher so that a single
// file can be validated here and executed there. This tool never executes any
// program; it only reads and analyzes policies.
//
// Usage:
//
//	palisade-lint lint    [--json] [--strict] <policy>...
//	palisade-lint inspect [--json] <policy>
//	palisade-lint help
//	palisade-lint version
//
// Exit codes:
//
//	0  success (no errors; also no warnings when --strict)
//	1  usage error
//	2  one or more policies had errors (or warnings under --strict)
//	3  a policy file could not be read or parsed
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/example/palisade/inspector/internal/lint"
	"github.com/example/palisade/inspector/internal/policy"
)

const version = "0.1.0"

const usage = `palisade-lint — inspect and lint palisade policy files

USAGE:
    palisade-lint lint    [--json] [--strict] <policy>...
    palisade-lint inspect [--json] <policy>
    palisade-lint help
    palisade-lint version

COMMANDS:
    lint      Validate one or more policies and print diagnostics.
    inspect   Print a structured summary of a single policy.
    help      Show this help.
    version   Print the version.

OPTIONS:
    --json     Emit machine-readable JSON instead of text.
    --strict   Treat warnings as failures (affects exit code).

This tool never executes programs; it only analyzes policy files.`

func main() {
	os.Exit(run(os.Args[1:]))
}

func run(args []string) int {
	if len(args) == 0 {
		fmt.Fprintln(os.Stderr, usage)
		return 1
	}
	switch args[0] {
	case "help", "-h", "--help":
		fmt.Println(usage)
		return 0
	case "version", "-V", "--version":
		fmt.Printf("palisade-lint %s\n", version)
		return 0
	case "lint":
		return cmdLint(args[1:])
	case "inspect":
		return cmdInspect(args[1:])
	default:
		fmt.Fprintln(os.Stderr, usage)
		fmt.Fprintf(os.Stderr, "palisade-lint: unknown command %q\n", args[0])
		return 1
	}
}

type flags struct {
	jsonOut bool
	strict  bool
	files   []string
}

func parseFlags(args []string) (flags, error) {
	var f flags
	for _, a := range args {
		switch a {
		case "--json":
			f.jsonOut = true
		case "--strict":
			f.strict = true
		default:
			if strings.HasPrefix(a, "-") {
				return f, fmt.Errorf("unknown option %q", a)
			}
			f.files = append(f.files, a)
		}
	}
	return f, nil
}

// fileReport pairs a filename with its lint report for JSON output.
type fileReport struct {
	File   string       `json:"file"`
	Report *lint.Report `json:"report"`
}

func cmdLint(args []string) int {
	f, err := parseFlags(args)
	if err != nil {
		fmt.Fprintf(os.Stderr, "palisade-lint: %v\n", err)
		return 1
	}
	if len(f.files) == 0 {
		fmt.Fprintln(os.Stderr, "palisade-lint: lint requires at least one policy file")
		return 1
	}

	var jsonReports []fileReport
	exit := 0

	for _, file := range f.files {
		text, err := os.ReadFile(file)
		if err != nil {
			fmt.Fprintf(os.Stderr, "palisade-lint: cannot read %q: %v\n", file, err)
			exit = 3
			continue
		}
		report, err := lint.Lint(string(text))
		if err != nil {
			// Parse errors are fatal for this file.
			if f.jsonOut {
				jsonReports = append(jsonReports, fileReport{
					File: file,
					Report: &lint.Report{Diagnostics: []lint.Diagnostic{
						{Severity: lint.Error, Code: "P000", Message: err.Error()},
					}},
				})
			} else {
				fmt.Printf("%s\n  error P000: %v\n", file, err)
			}
			exit = maxInt(exit, 2)
			continue
		}

		if f.jsonOut {
			jsonReports = append(jsonReports, fileReport{File: file, Report: report})
		} else {
			printTextReport(file, report)
		}

		errs, warns, _ := report.Counts()
		if errs > 0 {
			exit = maxInt(exit, 2)
		}
		if f.strict && warns > 0 {
			exit = maxInt(exit, 2)
		}
	}

