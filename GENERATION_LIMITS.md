# Generation Limits and Customization (Ограничения и кастомизация генерации)

This document describes the new parameters added to control fairy tale generation in FableForge.

## Overview

The following new CLI parameters have been added to provide fine-grained control over:
- Setting/place descriptions
- Character names
- Episode and moment limits
- Total character count limits

These parameters work with both random generation and formula-based generation.

## CLI Parameters

### Place and Time Settings

Specify the setting location and time period for the fairy tale:

```bash
# Place description
fableforge generate --place "kingdom by the sea"
fableforge generate --place "футуристический мегаполис"

# Time period
fableforge generate --time "ancient times"
fableforge generate --time "год 2099"

# Both together
fableforge generate --place "космическая станция" --time "далёкое будущее"
```

**How it works:**
- Sets `Tale.initial.setting.place` and `Tale.initial.setting.time`
- These descriptions are used by the LLM when generating the story
- Works even if `--from-formula` is used

### Character Names

Pre-assign names to characters instead of letting the LLM generate them:

```bash
# One character
fableforge generate --name "Иван"

# Multiple characters (assigned in order by PersonaId)
fableforge generate --name "Иван" --name "Баба-Яга" --name "Змей Горыныч"

# Modern names for modern settings
fableforge generate --genre detective --name "Detective Miller" --name "Professor Chen"
```

**How it works:**
- Names are assigned to `Persona.attributes["name"]` based on PersonaId order
- If fewer names are provided than characters, remaining characters get LLM-generated names
- Names are used by the LLM when generating the narrative

### Episode Limits

Control how many episodes are generated:

```bash
# Limit to 5 episodes total
fableforge generate --moves 2 --max-episodes 5

# Combine with character limit
fableforge generate --max-episodes 3 --max-characters 5000
```

**How it works:**
- Episodes are created by segmenting the tale by narrative phases
- `--max-episodes` truncates the episode list after structure generation
- Character generation and initial situation count as episodes
- Useful for getting shorter stories or controlling generation cost

### Moments Per Episode

Control narrative density by limiting moments (narrative functions) per episode:

```bash
# Maximum 3 moments per episode
fableforge generate --max-moments-per-episode 3

# Sparse narrative (1-2 moments per episode)
fableforge generate --max-moments-per-episode 2
```

**How it works:**
- Each episode contains moments (instances of narrative functions)
- Higher moment counts = denser, more complex episodes
- Lower moment counts = simpler, more focused episodes
- Applied after episodes are segmented but before LLM generation

### Maximum Character Count

Limit the total length of generated fairy tale text:

```bash
# Short story (~2000 characters)
fableforge generate --max-characters 2000

# Medium story (~5000 characters)
fableforge generate --max-characters 5000

# Novel-length (~50000 characters)
fableforge generate --max-characters 50000
```

**How it works:**
- Checks accumulated text length after each episode is generated
- Stops generation when limit is reached
- Allows natural stopping points (between episodes)
- Final text may be slightly over limit due to episode boundaries

## Configuration Objects

### GenConfig (fableforge-core)

New fields added:

```rust
pub struct GenConfig {
    // ... existing fields ...

    /// Maximum number of episodes (optional limit for LLM generation).
    pub max_episodes: Option<usize>,

    /// Maximum moments per episode (optional limit for LLM generation).
    pub max_moments_per_episode: Option<usize>,
}
```

Builder methods:

```rust
let config = GenConfig::new()
    .with_max_episodes(5)
    .with_max_moments_per_episode(3);
```

### StoryComposer (fableforge-llm)

New fields added:

```rust
pub struct StoryComposer<C: LlmClient> {
    // ... existing fields ...

    /// Maximum character count for generated story (optional).
    max_characters: Option<usize>,

    /// Maximum number of episodes to generate (optional).
    max_episodes: Option<usize>,

    /// Maximum moments per episode (optional).
    max_moments_per_episode: Option<usize>,
}
```

Builder methods:

```rust
let composer = StoryComposer::new(client, style)
    .with_max_characters(5000)
    .with_max_episodes(5)
    .with_max_moments_per_episode(3);
```

### Persona (fableforge-core)

Convenience methods for character names:

```rust
// Set name
let persona = Persona::new(1, vec![Sphere::Hero])
    .with_name("Иван");

// Get name
if let Some(name) = persona.name() {
    println!("Character: {}", name);
}
```

## Examples

### Detective Story with Custom Characters

```bash
fableforge generate \
  --genre "detective noir" \
  --setting "1940s New York" \
  --tone "dark, cynical" \
  --name "Detective Mike Hammer" \
  --name "Femme Fatale Velma" \
  --place "rainy Manhattan streets" \
  --time "winter 1947" \
  --max-characters 8000 \
  --max-episodes 6
```

### Short Sci-Fi Tale

```bash
fableforge generate \
  --genre "hard sci-fi" \
  --era "distant future" \
  --place "generation ship Aniara" \
  --time "year 2847" \
  --max-characters 3000 \
  --max-moments-per-episode 2
```

### Classic Russian Fairy Tale

```bash
fableforge generate \
  --genre "русская сказка" \
  --name "Иван-царевич" \
  --name "Василиса Прекрасная" \
  --place "тридевятое царство" \
  --time "в стародавние времена" \
  --moves 2
```

### Minimalist Horror

```bash
fableforge generate \
  --genre "psychological horror" \
  --tone "unsettling, minimalist" \
  --max-episodes 3 \
  --max-moments-per-episode 2 \
  --max-characters 2500
```

## Technical Notes

### Episode Segmentation

Episodes are created by grouping narrative moments by Propp's phases:
1. **Character Generation** - always first episode
2. **Initial Situation** - if tale has initial situation
3. **Preparation** - Absentation, Interdiction, Violation, etc.
4. **Complication** - Villainy or Lack
5. **Donor** - DonorTest, HeroReaction, Acquisition
6. **Struggle** - Guidance, Struggle, Victory
7. **Return** - Return, Pursuit, Rescue
8. **Recognition** - UnrecognizedArrival, Recognition, Exposure
9. **Resolution** - Transfiguration, Punishment, HappyEnding

Limits are applied after segmentation.

### Character Count vs Episode Count

Both limits work together:
- **Episode limit**: Structural control - controls narrative arc length
- **Character limit**: Text length control - controls physical story length
- When both are set, whichever limit is hit first stops generation

### Cost Management

Use limits to manage LLM API costs:
- `--max-episodes 3` - Fewer API calls
- `--max-characters 2000` - Shorter responses (less token usage)
- `--max-moments-per-episode 2` - Simpler prompts

### Reproducibility

When using `--seed`, limits don't affect structural determinism:
- Same seed → same structure before limits
- Limits only affect **what gets generated**, not **what structure exists**

## Migration Notes

All new parameters are optional and backward-compatible:
- Existing commands work unchanged
- No breaking changes to Tale or GenConfig structs
- Setting/time were already in Tale structure, now exposed in CLI
- Names use existing Persona.attributes HashMap
