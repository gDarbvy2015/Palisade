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
	cargo build --release --manifest-path $(LAUNCHER_DIR)/Cargo.toml

build-go:
	cd $(INSPECTOR_DIR) && go build ./cmd/palisade-lint

## test: run all tests for both languages
test: test-rust test-go

test-rust:
	cargo test --manifest-path $(LAUNCHER_DIR)/Cargo.toml

test-go:
	cd $(INSPECTOR_DIR) && go test ./...

## fmt: format both codebases
fmt:
	cargo fmt --manifest-path $(LAUNCHER_DIR)/Cargo.toml
	cd $(INSPECTOR_DIR) && gofmt -w .

## vet: static analysis (clippy if present, plus go vet)
vet:
	cargo clippy --manifest-path $(LAUNCHER_DIR)/Cargo.toml --all-targets || true
	cd $(INSPECTOR_DIR) && go vet ./...

## lint-examples: run palisade-lint over the example policies
lint-examples: build-go
	$(LINT_BIN) lint examples/echo.policy examples/git-status.policy examples/network-fetch.policy

## clean: remove build artifacts
clean:
	cargo clean --manifest-path $(LAUNCHER_DIR)/Cargo.toml || true
	rm -f $(INSPECTOR_DIR)/palisade-lint $(INSPECTOR_DIR)/palisade-lint.exe

## help: list targets
help:
	@grep -E '^## ' $(MAKEFILE_LIST) | sed 's/## //'
