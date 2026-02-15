# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

FableForge — a Propp-morphology-based fairy tale generator. Combines algorithmic structure generation (Vladimir Propp's 32 narrative functions) with LLM-based narrative text generation via Claude API.

## Build & Test Commands

```bash
cargo build                              # build CLI (default member)
cargo build --workspace                  # build everything
cargo test --workspace                   # run all tests (119 tests)
cargo test -p fableforge-core            # test core crate only
cargo test -p fableforge-llm             # test LLM crate only
cargo test -p fableforge-core -- syntax  # run specific test(s) by name
cargo run -- structure --moves 2         # show morphological structure
cargo run -- generate --api-key $KEY     # generate a full tale
```

Rust edition 2024 — requires nightly toolchain.

## Architecture

**Workspace layout:** 4 crates.

```
cli/fableforge (binary) → fableforge-llm → fableforge-core
crates/fableforge-tg (binary) → fableforge-llm → fableforge-core
```

### fableforge-core

Morphological engine. No network, no async — pure data structures and algorithms.

- **function.rs** — 32 narrative functions (Absentation, Villainy, etc.) organized into 7 phases
- **subtype.rs** — 168 function subtypes (e.g., Villainy has 19: kidnapping, theft, destruction...)
- **dramatis.rs** — 7 character spheres of action (Hero, Villain, Donor, Helper, Princess, Dispatcher, FalseHero)
- **tale.rs** — Tale = optional InitialSituation + Vec<Move>. Moves contain Moments linking functions to agents/patients
- **generate.rs** — `RandomGen` (probabilistic) and `TemplateGen` (predefined patterns) both implement `Generator` trait. RandomGen supports negation of functions, coherent subtype dependencies (Villainy→Liquidation, Branding→Recognition), and two variants of embedded moves (Lack-based and Villainy-based)
- **syntax.rs** — Validates function sequences against Propp's rules, computes absurdity score (0.0–1.0)
- **formula.rs** — Parses/serializes Propp's symbolic notation (e.g., "α β¹ γ A¹ ↑") using winnow
- **connective.rs** — Narrative connectives between functions (Motivation, Transference, Temporal, Custom)

### fableforge-llm

Claude API integration. Transforms tale structure into prompts, collects responses, assembles narrative.

- **client.rs** — `LlmClient` trait + `ClaudeClient` + `OpenRouterClient` (with exponential backoff retry) + `MockClient` for tests
- **composer.rs** — `StoryComposer` orchestrates two-phase generation: characters first, then episodes
- **episode.rs** — Segments tale into episodes grouped by Propp's phases
- **prompt.rs** — `PromptBuilder` + `StyleConfig` (genre, setting, tone, era, custom instructions). Prompts are in Russian
- **context.rs** — `TaleContext` maintains state across episodes (characters, setting, previous summaries)
- **evaluate.rs** — `CoherenceEvaluator` scores generated tales on multiple dimensions via LLM
- **error.rs** — Error types for LLM operations

### fableforge-tg

Telegram bot. Interactive fairy tale generation via inline keyboards (genre → tone → moves → seed flow).

- **handlers.rs** — Dialogue FSM (`BotState`), callback handlers, generation logic
- **format.rs** — Telegram message formatting
- **state.rs** — Bot state and dialogue management

### cli/fableforge

Two subcommands: `generate` (full tale with LLM) and `structure` (morphological outline only).

## Key Design Decisions

- **Bilingual**: All functions/phases have both English and Russian names via `Lang` enum. LLM prompts are Russian-language
- **Reproducibility**: Seeded RNG for deterministic structure generation
- **LLM abstraction**: `LlmClient` trait allows swapping providers; `MockClient` enables testing without API calls
- **Two-phase LLM generation**: Characters generated in a single prompt first (for consistency), then episodes sequentially with accumulated context
- **Absurdity scoring**: Generated tales are validated against canonical Propp rules; structures exceeding `max_absurdity` threshold are rejected and regenerated
- **Function negation**: Interdiction, HeroReaction, Struggle, Rescue, Punishment can be negated with cascading effects (e.g., neg-Struggle skips Branding)
- **Coherent subtypes**: Villainy subtype determines Liquidation subtype (kidnapping→captive freed, spell→spell broken); Branding subtype determines Recognition subtype
