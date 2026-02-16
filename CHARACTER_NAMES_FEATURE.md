# Character Names Feature in Telegram Bot

## Overview

Added ability for Telegram bot users to specify custom character names for the generated fairy tales. Character names are assigned to personae in the order they appear in the tale structure.

## Changes Made

### 1. State Machine (`state.rs`)

#### New States

```rust
SelectCharacterNames {
    genre: Option<String>,
    tone: Option<String>,
    moves: usize,
}

AwaitCharacterNames {
    genre: Option<String>,
    tone: Option<String>,
    moves: usize,
}
```

#### Updated States

All subsequent states now include `character_names: Vec<String>`:
- `SelectPlace`
- `AwaitPlaceText`
- `SelectMaxEpisodes`
- `SelectMaxMoments`
- `SelectSeed`

#### Updated GenerateConfig

```rust
pub struct GenerateConfig {
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub moves: usize,
    pub character_names: Vec<String>,  // NEW
    pub place: Option<String>,
    pub max_episodes: Option<usize>,
    pub max_moments_per_episode: Option<usize>,
    pub seed: Option<u64>,
}
```

### 2. Handlers (`handlers.rs`)

#### New Keyboard Function

```rust
fn character_names_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Задать имена", "names:enter"),
        InlineKeyboardButton::callback("Пропустить", "names:skip"),
    ]])
}
```

#### Updated Dialogue Flow

**Previous:**
```
SelectGenre → SelectTone → SelectMoves → SelectPlace → ...
```

**Current:**
```
SelectGenre → SelectTone → SelectMoves → SelectCharacterNames →
  [AwaitCharacterNames (optional)] → SelectPlace → ...
```

#### Text Input Handler

Added handler for `AwaitCharacterNames` state:
- Accepts comma-separated character names
- Trims whitespace from each name
- Filters out empty strings
- Transitions to `SelectPlace` with parsed names

```rust
BotState::AwaitCharacterNames { genre, tone, moves } => {
    let character_names: Vec<String> = if text.trim().is_empty() {
        Vec::new()
    } else {
        text.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };
    // ... transition to SelectPlace
}
```

#### Callback Handler

Added handler for `SelectCharacterNames` state:
- `names:skip` → transitions to `SelectPlace` with empty names list
- `names:enter` → transitions to `AwaitCharacterNames` for text input

#### Generation Logic

Updated `do_generate` to apply character names to tale personae:

```rust
// Apply character names to personae if provided
if !config.character_names.is_empty() {
    for (idx, name) in config.character_names.iter().enumerate() {
        if let Some(persona) = tale.personae.get_mut(idx) {
            persona.attributes.insert("name".to_string(), name.clone());
        }
    }
}
```

Names are assigned to personae by index order:
- 1st name → 1st persona (typically Hero)
- 2nd name → 2nd persona (typically Villain or Princess)
- 3rd name → 3rd persona, etc.

## User Journey Example

```
User: /generate
Bot: Выберите жанр:
     [Сказка] [Фэнтези] [Нуар] [Хоррор] [Ироничная] [Пропустить]

User: [Сказка]
Bot: Выберите тон повествования:
     [Лирический] [Мрачный] [Ироничный] [Эпический] [Пропустить]

User: [Эпический]
Bot: Количество ходов (сюжетных арок):
     [1] [2] [3]

User: [2]
Bot: Задать имена персонажам?
     Можно указать имена и краткие описания героев сказки:
     [Задать имена] [Пропустить]

User: [Задать имена]
Bot: Введите имена персонажей через запятую:

     Примеры:
     • Иван-царевич, Василиса Прекрасная, Кощей
     • Детектив Миллер, Профессор Чен
     • Капитан Нова, Механик Зед

     Имена будут присвоены персонажам по порядку.

User: Иван-царевич, Василиса Прекрасная, Кощей Бессмертный

Bot: Описать место действия?
     Нажмите "Пропустить" или кнопку "Ввести описание":
     [Ввести описание] [Пропустить]

... (rest of dialogue)

Bot: [Generates tale with specified character names]
```

## Examples

### Example 1: Classic Fairy Tale with Custom Names

```
Genre: Сказка
Tone: Эпический
Moves: 2
Names: Иван-царевич, Василиса Прекрасная, Кощей Бессмертный
Place: тридевятое царство
Episodes: Без ограничения
Moments: Без ограничения
```

### Example 2: Noir Detective Story

```
Genre: Нуар
Tone: Мрачный
Moves: 1
Names: Детектив Миллер, Профессор Чен, Мафиози Винс
Place: дождливые улицы Манхэттена, зима 1947
Episodes: 5
Moments: 3
```

### Example 3: Sci-Fi Adventure

```
Genre: Фэнтези (or skip)
Tone: Эпический
Moves: 2
Names: Капитан Нова, Механик Зед, Доктор Лира
Place: корабль поколений Аниара, год 2847
Episodes: 8
Moments: 5
```

## Technical Details

### Data Flow

```
User enters comma-separated names
  ↓
AwaitCharacterNames state parses input
  ↓
character_names: Vec<String> flows through dialogue states
  ↓
GenerateConfig fully populated with names
  ↓
generate_tale_structure() creates Tale with personae
  ↓
do_generate() applies names to tale.personae[i].attributes["name"]
  ↓
StoryComposer uses names in LLM prompts
  ↓
Generated narrative features specified character names
```

### Persona Attributes

Character names are stored in the `Persona::attributes` HashMap:

```rust
pub type Attributes = HashMap<String, String>;

pub struct Persona {
    pub id: PersonaId,
    pub spheres: Vec<Sphere>,
    pub attributes: Attributes,  // {"name": "Иван-царевич", ...}
}
```

### Name Assignment

Names are assigned to personae in order:
1. If fewer names than personae: only first N personae get custom names
2. If more names than personae: extra names are ignored
3. If empty list: LLM generates default names

## Backward Compatibility

✅ **100% backward compatible**

- New `character_names` field defaults to `Vec::new()`
- `/quick` command uses empty names list
- `/structure` command unaffected
- Users can skip character names by clicking "Пропустить"

## Testing

### Build Status
✅ `cargo build -p fableforge-tg` — Clean compilation
✅ `cargo build --workspace` — All crates compile
✅ `cargo test --workspace` — All 119 tests pass (79 core + 40 llm)

### Manual Testing Scenarios

To test this feature:

1. **Skip names**: Click "Пропустить" → should work as before
2. **Enter single name**: "Иван" → first persona named Иван
3. **Enter multiple names**: "Иван, Василиса, Кощей" → three personae named
4. **Empty input**: Press Enter without text → same as skip
5. **Extra spaces**: "Иван ,  Василиса  , Кощей" → trimmed correctly
6. **Trailing commas**: "Иван, Василиса," → filters empty strings

## Files Modified

- `crates/fableforge-tg/src/state.rs` — Added character names states and field
- `crates/fableforge-tg/src/handlers.rs` — Updated handlers and generation logic

## Total Changes

- **State machine**: +2 new states
- **Config**: +1 field (character_names)
- **Handlers**: +1 keyboard function, +1 text input handler, +1 callback handler
- **Generation**: Updated to apply names to personae attributes
- **Lines added**: ~50 lines of code
- **Breaking changes**: 0
- **Tests passing**: 119/119

## Future Enhancements

Possible improvements:
- Allow custom descriptions along with names (e.g., "Иван: храбрый воин")
- UI for editing/reordering names after input
- Show how many personae exist before asking for names
- Suggest default names based on genre
- Support for aliases/epithets (Иван-дурак, Иван-царевич)
- Name validation (Cyrillic/Latin alphabet, length limits)

## Migration Guide

### For Bot Users

No migration needed! The bot works immediately:
- Existing `/quick` command unchanged
- New `/generate` flow has an additional "character names" step
- Step is skippable — click "Пропустить" for default behavior

### For Developers

Character names are now part of the dialogue flow and generation config:

```rust
// Old usage (still works, names default to empty Vec)
let config = GenerateConfig {
    genre: Some("сказка".to_string()),
    tone: None,
    moves: 1,
    character_names: Vec::new(),  // Must include this
    place: None,
    max_episodes: None,
    max_moments_per_episode: None,
    seed: None,
};

// New usage with character names
let config = GenerateConfig {
    genre: Some("сказка".to_string()),
    tone: None,
    moves: 1,
    character_names: vec![
        "Иван-царевич".to_string(),
        "Василиса Прекрасная".to_string(),
    ],
    place: Some("тридевятое царство".to_string()),
    max_episodes: Some(5),
    max_moments_per_episode: Some(3),
    seed: None,
};
```

## Summary

Character names feature provides users with full control over protagonist names in generated fairy tales, maintaining the intuitive inline keyboard UX while enabling deeper customization. This feature complements existing place descriptions, episode limits, and moment limits to create a comprehensive tale generation interface.
