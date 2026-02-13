# FableForge

Propp-morphology-based fairy tale generator. Combines algorithmic structure generation ([Vladimir Propp's 31 narrative functions](https://en.wikipedia.org/wiki/Vladimir_Propp#Morphology_of_the_Folk_Tale)) with LLM-based narrative text generation via the Claude API.

## How it works

1. **Structure generation** — builds a morphologically valid fairy tale skeleton: initial situation, character spheres of action (Hero, Villain, Donor, Helper, Princess, Dispatcher, False Hero), and a sequence of narrative functions (Absentation, Villainy, Departure, Struggle, Victory, etc.) organized into moves and phases.
2. **Validation** — checks function sequences against Propp's canonical rules and computes an absurdity score (0.0–1.0). Structures exceeding the threshold are rejected and regenerated.
3. **Text generation** — two-phase LLM pipeline: characters are generated first (for consistency), then episodes are produced sequentially with accumulated context.

## Usage

Requires Rust nightly (see `rust-toolchain.toml`).

```bash
cargo build
```

### Show morphological structure

```bash
cargo run -- structure --moves 2
cargo run -- structure --moves 1 --seed 42    # reproducible
```

### Generate a full tale

```bash
export ANTHROPIC_API_KEY="sk-..."

cargo run -- generate --moves 2
cargo run -- generate --genre horror --tone dark --setting "abandoned castle"
cargo run -- generate --era "19th century" --genre detective --model claude-opus-4-20250514
```

#### Style options

| Flag | Description | Examples |
|------|-------------|----------|
| `-g, --genre` | Genre | detective, horror, fantasy, drama |
| `-s, --setting` | Setting | medieval castle, space station, modern city |
| `-t, --tone` | Narrative tone | dark, ironic, lyrical, epic |
| `-e, --era` | Era | ancient times, 19th century, future |
| `-c, --custom` | Custom instructions | free-form text |
| `-M, --model` | Claude model | claude-sonnet-4-20250514 (default) |

### Docker

```bash
docker build -f docker/Dockerfile -t fableforge .
docker run --rm fableforge structure --moves 1
docker run --rm -e ANTHROPIC_API_KEY fableforge generate --moves 1
```

## Architecture

Three crates with a clear dependency chain:

```
cli/fableforge (binary) → fableforge-llm → fableforge-core
```

**fableforge-core** — morphological engine. Pure data structures and algorithms, no network, no async. 32 narrative functions, 168 subtypes, 7 character spheres, sequence validation, Propp's symbolic notation parser (`α β¹ γ A¹ ↑`).

**fableforge-llm** — Claude API integration. `LlmClient` trait for provider abstraction, `StoryComposer` for two-phase generation, `PromptBuilder` with `StyleConfig`, episode segmentation by Propp's phases.

**cli/fableforge** — CLI with two subcommands: `generate` and `structure`.

## Testing

```bash
cargo test --workspace            # all tests
cargo test -p fableforge-core     # core only
cargo test -p fableforge-llm      # LLM only (uses MockClient)
```

## License

See [LICENSE](LICENSE).
