# FableForge

Propp-morphology-based fairy tale generator. Combines algorithmic structure generation ([Vladimir Propp's 32 narrative functions](https://en.wikipedia.org/wiki/Vladimir_Propp#Morphology_of_the_Folk_Tale)) with LLM-based narrative text generation via the Claude API.

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
# Using OpenRouter (recommended - supports multiple providers)
export OPENROUTER_API_KEY="sk-or-v1-..."

cargo run -- generate --provider openrouter --moves 2
cargo run -- generate --provider openrouter --genre horror --tone dark --setting "abandoned castle"
cargo run -- generate --provider openrouter --model anthropic/claude-opus-4

# Using Anthropic API directly
export ANTHROPIC_API_KEY="sk-ant-..."

cargo run -- generate --moves 2
cargo run -- generate --genre detective --era "19th century"
```

#### Style options

| Flag | Description | Examples |
|------|-------------|----------|
| `-g, --genre` | Genre | detective, horror, fantasy, drama |
| `-s, --setting` | Setting | medieval castle, space station, modern city |
| `-t, --tone` | Narrative tone | dark, ironic, lyrical, epic |
| `-e, --era` | Era | ancient times, 19th century, future |
| `-c, --custom` | Custom instructions | free-form text |
| `-p, --provider` | LLM provider | openrouter (recommended), claude |
| `-M, --model` | Model name | anthropic/claude-sonnet-4 (openrouter), claude-sonnet-4-20250514 (anthropic) |

### Docker

```bash
docker build -f docker/Dockerfile -t fableforge .
docker run --rm fableforge structure --moves 1

# With OpenRouter (recommended)
docker run --rm -e OPENROUTER_API_KEY fableforge generate --provider openrouter --moves 1

# With Anthropic API
docker run --rm -e ANTHROPIC_API_KEY fableforge generate --moves 1

# Telegram bot (interactive generation with place descriptions, episode/moment limits)
docker build -f docker/Dockerfile.tg -t fableforge-tg .
docker run --rm -e TELOXIDE_TOKEN -e OPENROUTER_API_KEY fableforge-tg
# See TELEGRAM_BOT_FEATURES.md for details on interactive dialogue
```

### Deploy to DigitalOcean App Platform

The Telegram bot can be deployed as a worker on [DigitalOcean App Platform](https://www.digitalocean.com/products/app-platform). The app spec lives in `.do/app.yaml`.

**One-time setup:**

1. Create the app:
   ```bash
   doctl apps create --spec .do/app.yaml
   ```
2. Note the app ID from the output.
3. Set encrypted env vars in the DO console: `TELOXIDE_TOKEN`, `OPENROUTER_API_KEY` (or `ANTHROPIC_API_KEY`).
4. Add GitHub secrets: `DIGITALOCEAN_ACCESS_TOKEN`, `DIGITALOCEAN_APP_ID`.

**CI/CD flow:** push to `main` → Docker workflow builds and pushes both `ghcr.io/vyorkin/fableforge` and `ghcr.io/vyorkin/fableforge-tg` images → deploy workflow triggers a redeployment on App Platform via `doctl`.

## Architecture

Four crates:

```
cli/fableforge (binary) → fableforge-llm → fableforge-core
crates/fableforge-tg (binary) → fableforge-llm → fableforge-core
```

**fableforge-core** — morphological engine. Pure data structures and algorithms, no network, no async. 32 narrative functions, 168 subtypes, 7 character spheres, function negation, coherent subtype dependencies, sequence validation, Propp's symbolic notation parser (`α β¹ γ A¹ ↑`).

**fableforge-llm** — LLM integration (Claude API, OpenRouter). `LlmClient` trait for provider abstraction, `StoryComposer` for two-phase generation, `PromptBuilder` with `StyleConfig`, episode segmentation by Propp's phases, coherence evaluation.

**fableforge-tg** — Telegram bot for interactive tale generation via inline keyboards. Supports place descriptions, episode/moment limits, and all style options. See [TELEGRAM_BOT_FEATURES.md](TELEGRAM_BOT_FEATURES.md) for details.

**cli/fableforge** — CLI with two subcommands: `generate` and `structure`.

## Testing

```bash
cargo test --workspace            # all tests
cargo test -p fableforge-core     # core only
cargo test -p fableforge-llm      # LLM only (uses MockClient)
```

## License

See [LICENSE](LICENSE).
