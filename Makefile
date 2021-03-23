# Palisade top-level Makefile.
#
# Targets orchestrate the Rust launcher (in launcher/) and the Go inspector
# (in inspector/). Works on Unix make and GNU make on Windows.

LAUNCHER_DIR := launcher
INSPECTOR_DIR := inspector

# Path to the built Go linter used by lint-examples.
ifeq ($(OS),Windows_NT)
LINT_BIN := $(INSPECTOR_DIR)/palisade-lint.exe
else
LINT_BIN := $(INSPECTOR_DIR)/palisade-lint
endif

.PHONY: all build build-rust build-go test test-rust test-go fmt vet \
        lint-examples clean help

all: build test

## build: compile both tools
build: build-rust build-go

build-rust:
