# Forge Specification

This directory contains the normative specifications and architectural decision records (ADRs) for the Forge framework.

## Structure

- [`PHILOSOPHY.md`](./PHILOSOPHY.md) — The foundational manifesto: why Forge exists and what it is trying to fix
- [`specs/`](./specs/) — Normative technical specifications defining language semantics, file formats, and protocols
- [`adrs/`](./adrs/) — Architectural Decision Records capturing the "why" behind foundational choices

## Reading Order

If you are new to the Forge codebase, read in this order:

1. `PHILOSOPHY.md` — understand the problem before the solution
2. `adrs/001` through `adrs/010` — understand why each major decision was made
3. The relevant spec(s) for the area you are working in

## ADR Status Legend

- **Proposed** — Under discussion, not yet implemented
- **Implemented** — In the codebase
- **Superseded** — Replaced by a later ADR (see the superseding ADR)
- **Deprecated** — No longer applicable
