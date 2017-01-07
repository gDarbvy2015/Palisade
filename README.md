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
