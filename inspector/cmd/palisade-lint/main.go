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

	if f.jsonOut {
		enc := json.NewEncoder(os.Stdout)
		enc.SetIndent("", "  ")
		if err := enc.Encode(jsonReports); err != nil {
			fmt.Fprintf(os.Stderr, "palisade-lint: json encode error: %v\n", err)
			return 3
		}
	}

	return exit
}

func printTextReport(file string, r *lint.Report) {
	errs, warns, infos := r.Counts()
	fmt.Printf("%s\n", file)
	if len(r.Diagnostics) == 0 {
		fmt.Println("  ok (no diagnostics)")
		return
	}
	for _, d := range r.Diagnostics {
		if d.Line > 0 {
			fmt.Printf("  %s %s (line %d): %s\n", d.Severity, d.Code, d.Line, d.Message)
		} else {
			fmt.Printf("  %s %s: %s\n", d.Severity, d.Code, d.Message)
		}
	}
	fmt.Printf("  summary: %d error(s), %d warning(s), %d info\n", errs, warns, infos)
}

// inspectView is the structured summary emitted by `inspect`.
type inspectView struct {
	File       string            `json:"file"`
	Name       string            `json:"name"`
	Program    string            `json:"program"`
	TimeoutMs  string            `json:"timeout_ms"`
	Workdir    string            `json:"workdir,omitempty"`
	EnvMode    string            `json:"env_mode"`
	EnvPass    []string          `json:"env_pass,omitempty"`
	EnvRemove  []string          `json:"env_remove,omitempty"`
	EnvSet     []string          `json:"env_set,omitempty"`
	FixedArgs  []string          `json:"fixed_args,omitempty"`
	AllowArgs  []string          `json:"allow_args,omitempty"`
	AllowExtra string            `json:"allow_extra"`
	PathRoots  []string          `json:"path_roots,omitempty"`
	NetMode    string            `json:"net_mode"`
	Keys       []string          `json:"declared_keys"`
	Diagnostic []lint.Diagnostic `json:"diagnostics"`
}

func cmdInspect(args []string) int {
	f, err := parseFlags(args)
	if err != nil {
		fmt.Fprintf(os.Stderr, "palisade-lint: %v\n", err)
		return 1
	}
	if len(f.files) != 1 {
		fmt.Fprintln(os.Stderr, "palisade-lint: inspect requires exactly one policy file")
		return 1
	}
	file := f.files[0]
	text, err := os.ReadFile(file)
	if err != nil {
		fmt.Fprintf(os.Stderr, "palisade-lint: cannot read %q: %v\n", file, err)
		return 3
	}
	doc, err := policy.Parse(string(text))
	if err != nil {
		fmt.Fprintf(os.Stderr, "palisade-lint: %v\n", err)
		return 3
	}
	report, err := lint.Lint(string(text))
	if err != nil {
		fmt.Fprintf(os.Stderr, "palisade-lint: %v\n", err)
		return 3
	}

	view := buildView(file, doc, report)

	if f.jsonOut {
		enc := json.NewEncoder(os.Stdout)
		enc.SetIndent("", "  ")
		if err := enc.Encode(view); err != nil {
			fmt.Fprintf(os.Stderr, "palisade-lint: json encode error: %v\n", err)
			return 3
		}
	} else {
		printInspect(view)
	}

	if report.HasErrors() {
		return 2
	}
	return 0
}

func buildView(file string, doc *policy.Document, report *lint.Report) inspectView {
	scalar := func(section, key, def string) string {
		if v, ok := doc.Scalar(section, key); ok {
			return v
		}
		return def
	}
	return inspectView{
		File:       file,
		Name:       scalar("", "name", "unnamed"),
		Program:    scalar("exec", "program", ""),
		TimeoutMs:  scalar("exec", "timeout_ms", "0 (none)"),
		Workdir:    scalar("exec", "workdir", ""),
		EnvMode:    scalar("env", "mode", "clean"),
		EnvPass:    doc.List("env", "pass"),
		EnvRemove:  doc.List("env", "remove"),
		EnvSet:     doc.List("env", "set"),
		FixedArgs:  doc.List("args", "fixed"),
		AllowArgs:  doc.List("args", "allow"),
		AllowExtra: scalar("args", "allow_extra", "false"),
		PathRoots:  doc.List("path", "root"),
		NetMode:    scalar("net", "mode", "none"),
		Keys:       doc.Keys(),
		Diagnostic: report.Diagnostics,
	}
}

func printInspect(v inspectView) {
