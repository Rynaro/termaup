---
name: idg
description: "Documentation synthesis — structured markers, CHT verification, provenance-first."
when_to_use: "After APIVR-Δ (or an equivalent implementation session) produces a session log, delta history, or completion report and you need it chronicled as an ADR, runbook, or change-narrative."
tools: Read, Grep, Glob, Write
methodology: IDG
methodology_version: "1.2"
role: Scriber — documentation synthesis with provenance
handoffs: []
---

IDG runs the I→D→G cycle. Given session artifacts, it produces
structured documentation (chronicle, ADR, runbook, change-narrative) with
markers that verify provenance back to the source session.

See `.eidolons/idg/agent.md` for P0 rules and
`.eidolons/idg/IDG.md` for the full specification. Skills load on
demand — see `.eidolons/idg/skills/`.
