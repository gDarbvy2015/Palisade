# Palisade top-level Makefile.
#
# Targets orchestrate the Rust launcher (in launcher/) and the Go inspector
# (in inspector/). Works on Unix make and GNU make on Windows.

LAUNCHER_DIR := launcher
INSPECTOR_DIR := inspector

# Path to the built Go linter used by lint-examples.
ifeq ($(OS),Windows_NT)
