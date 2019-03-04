<!-- PALISADE — FORTRESS OPERATIONS HANDBOOK -->

<div align="center">

<img src="docs/assets/gatehouse.svg" alt="The Palisade gatehouse: a command must clear five wardens — program, args, env, path, time — before the copper portcullis rises and admits it" width="100%"/>

# Palisade — Fortress Operations Handbook

**A cross-platform, userspace declarative command policy launcher.**
*You post the standing orders once; the gatehouse enforces them the same way, everywhere.*

</div>

> [!WARNING]
> **This is a gatehouse, not a dungeon.** Palisade is a **userspace launcher, not a kernel
> sandbox.** It decides *whether and how* a command is launched. It does **not** confine a
> process once running. Read [§7 · The Threat Boundary](#7--the-threat-boundary) first.

---

## How to read this handbook

Not a feature brochure — a duty roster, written the way you would brief an operator taking a
shift at the gate. Read in order the first time; return by number afterwards.

**Posts:**
[1 Operator Briefing](#1--operator-briefing) ·
[2 Muster](#2--muster--build-and-install) ·
[3 The Two Wardens](#3--the-two-wardens) ·
[4 Standing Orders](#4--reading-standing-orders--policy-anatomy) ·
[5 Admission Sequence](#5--the-admission-sequence) ·
[6 Seal of Record](#6--the-seal-of-record--receipts) ·
[7 Threat Boundary](#7--the-threat-boundary) ·
[8 Field Transcripts](#8--field-transcripts) ·
[9 Armoury](#9--recipes-from-the-armoury) ·
[10 Failure Diagnosis](#10--failure-diagnosis) ·
[11 Cross-Platform](#11--cross-platform-notes) ·
[12 Limitations & Roadmap](#12--limitations--roadmap)

---

## 1 · Operator Briefing

Ad-hoc command invocation is where accidents live. A wrapper script leaks the full
environment to a subprocess. A CI step quietly accepts an extra `--force`. A cron job with
no timeout hangs a runner. Each is an *implicit* assumption nobody reviewed.

Palisade turns those assumptions into a **standing order** — a small, auditable policy file
that describes exactly how one command may run: its **executable** (a bare name via `PATH`,
or an explicit path); its **arguments** (a fixed prefix plus an allow-list of what a caller
may add); its **environment** (started clean and populated, or inherited and pruned); an
optional **path constraint** on which binary launches; a declared **network intent**
(advisory — see [§7](#7--the-threat-boundary)); and a wall-clock **timeout**.

The launcher then resolves the executable, filters the environment, validates the arguments,
spawns the process, enforces the timeout, and writes a machine-readable **receipt** of what
actually happened. Lint the file, diff it in review, and run it identically on a laptop, a CI
runner, and a cron host.

Use Palisade to make command execution **explicit, consistent, and auditable** — in CI
wrappers, cron jobs, and operator runbooks. Do **not** use it as a barrier against hostile
code. For real isolation, station the gatehouse *inside* a container, VM, or OS sandbox.

---

## 2 · Muster — build and install

Two tools garrison the fortress, built from two toolchains — both **standard-library only**,
no third-party dependencies. **Prerequisites:** Rust (1.74+) and Go (1.24+).

```sh
# The launcher (Rust) — the tool that executes.
cd launcher && cargo build --release        # -> launcher/target/release/palisade

# The inspector (Go) — the tool that only reads.
cd inspector && go build ./cmd/palisade-lint # -> palisade-lint(.exe)
```

Or muster both at once through the top-level [`Makefile`](Makefile):
`make build` (compile both), `make test` (cargo test + go test), `make lint-examples`,
`make fmt`, `make vet`.

---

## 3 · The two wardens

One fortress format; two independently-written tools that must agree on it. The grammar is
implemented **twice** — once in Rust, once in Go — and cross-checked by each tool's tests.

| Warden | Language | Standing duty | Executes? |
|--------|----------|---------------|:---------:|
| [`palisade`](launcher/) | Rust (stdlib) | Loads + validates a policy, resolves the program, **runs** it, writes a receipt. | **Yes** |
| [`palisade-lint`](inspector/) | Go (stdlib) | **Inspects and lints** the same format. Reads files, reports diagnostics. | **Never** |

The division is deliberate. The **inspector never launches a process** — run it safely
against an untrusted policy to learn what it *would* do. The **launcher** is the only tool
with the keys to the gate. Both read the same normative grammar in
[`docs/POLICY.md`](docs/POLICY.md).

---
