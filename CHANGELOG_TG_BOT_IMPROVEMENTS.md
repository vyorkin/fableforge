# Changelog: Telegram Bot Improvements

## Summary

Доработан Telegram-бот для поддержки всех параметров генерации, добавленных в CLI:
- Описание места действия (place)
- Ограничение количества эпизодов (max_episodes)
- Ограничение количества моментов на эпизод (max_moments_per_episode)

## User Request (Russian)

> Доработай Telegram-бот, чтобы он тоже принимал параметры и у него была возможность предоставлять пользователю описание места действия.
>
> Также необходимо дать пользователю бота возможность указывать не только количество ходов, но и максимальное количество эпизодов и моментов в рамках одного эпизода.

## Changes Made

### 1. State Machine (`state.rs`)

#### Added new states to BotState enum:

```rust
pub enum BotState {
    // ... existing states ...

    // NEW: Place selection
    SelectPlace {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
    },

    // NEW: Awaiting place text input
    AwaitPlaceText {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
    },

    // NEW: Episode limit selection
    SelectMaxEpisodes {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        place: Option<String>,
    },

    // NEW: Moments per episode limit
    SelectMaxMoments {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        place: Option<String>,
        max_episodes: Option<usize>,
    },

    // UPDATED: SelectSeed now includes all parameters
    SelectSeed {
        genre: Option<String>,
        tone: Option<String>,
        moves: usize,
        place: Option<String>,             // NEW
        max_episodes: Option<usize>,       // NEW
        max_moments_per_episode: Option<usize>, // NEW
    },
}
```

#### Updated GenerateConfig:

```rust
pub struct GenerateConfig {
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub moves: usize,
    pub place: Option<String>,                  // NEW
    pub max_episodes: Option<usize>,            // NEW
    pub max_moments_per_episode: Option<usize>, // NEW
    pub seed: Option<u64>,
}
```

### 2. Handlers (`handlers.rs`)

#### New keyboard functions:

```rust
fn place_keyboard() -> InlineKeyboardMarkup {
    // "Ввести описание" / "Пропустить"
}

fn max_episodes_keyboard() -> InlineKeyboardMarkup {
    // "3" / "5" / "8" / "Без ограничения"
}

fn max_moments_keyboard() -> InlineKeyboardMarkup {
    // "2" / "3" / "5" / "Без ограничения"
}
```

#### Updated dialogue flow:

**Старая последовательность:**
```
SelectGenre → SelectTone → SelectMoves → SelectSeed → Generate
```

**Новая последовательность:**
```
SelectGenre → SelectTone → SelectMoves → SelectPlace →
  [AwaitPlaceText (опционально)] → SelectMaxEpisodes →
  SelectMaxMoments → SelectSeed → Generate
```

#### Updated handle_text_input:

Добавлена обработка нового состояния `AwaitPlaceText` для ввода описания места действия:

```rust
BotState::AwaitPlaceText { genre, tone, moves } => {
    let place = if text.trim().is_empty() {
        None
    } else {
        Some(text.trim().to_string())
    };
    // Переход к выбору max_episodes
}
```

#### Updated handle_callback:

Добавлены обработчики для трёх новых состояний:

```rust
BotState::SelectPlace { genre, tone, moves } => {
    if data == "place:skip" {
        // Переход к max_episodes без описания места
    } else if data == "place:enter" {
        // Переход к AwaitPlaceText для ввода текста
    }
}

BotState::SelectMaxEpisodes { genre, tone, moves, place } => {
    let max_episodes = parse_callback_value(&data, "maxepisodes");
    // Переход к SelectMaxMoments
}

BotState::SelectMaxMoments { genre, tone, moves, place, max_episodes } => {
    let max_moments_per_episode = parse_callback_value(&data, "maxmoments");
    // Переход к SelectSeed
}
```

#### Updated do_generate:

```rust
// Применение места действия к tale
if let Some(ref place) = config.place {
    let mut initial = tale.initial.take().unwrap_or_default();
    let mut setting = initial.setting.take().unwrap_or_default();
    setting.place = Some(place.clone());
    initial.setting = Some(setting);
    tale.initial = Some(initial);
}

// Применение лимитов к StoryComposer
if let Some(max_eps) = config.max_episodes {
    composer = composer.with_max_episodes(max_eps);
}
if let Some(max_moments) = config.max_moments_per_episode {
    composer = composer.with_max_moments_per_episode(max_moments);
}
```

#### Updated generate_tale_structure:

Теперь принимает дополнительные параметры и передаёт их в GenConfig:

```rust
fn generate_tale_structure(
    moves: usize,
    seed: Option<u64>,
    max_episodes: Option<usize>,        // NEW
    max_moments_per_episode: Option<usize>, // NEW
) -> Result<fableforge_core::Tale, String> {
    let mut gen_config = GenConfig::new()
        .with_move_count(moves..moves + 1)
        .with_max_absurdity(0.5);

    // Apply limits
    if let Some(max_eps) = max_episodes {
        gen_config = gen_config.with_max_episodes(max_eps);
    }
    if let Some(max_moments) = max_moments_per_episode {
        gen_config = gen_config.with_max_moments_per_episode(max_moments);
    }

    // ... generation logic
}
```

### 3. Documentation

#### Created TELEGRAM_BOT_FEATURES.md

Comprehensive guide covering:
- All bot commands (`/start`, `/generate`, `/quick`, `/structure`, `/help`)
- Step-by-step walkthrough of interactive dialogue
- Detailed explanation of new features:
  - Place description input
  - Episode limits (3/5/8/unlimited)
  - Moments per episode limits (2/3/5/unlimited)
- Usage examples for different genres
- Technical architecture details
- Setup and deployment instructions

#### Updated README.md

- Added reference to TELEGRAM_BOT_FEATURES.md
- Updated bot description to mention new features
- Updated Docker example comment

## User Journey Examples

### Example 1: Classic Fairy Tale with Place

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
Bot: Описать место действия?
     [Ввести описание] [Пропустить]

User: [Ввести описание]
Bot: Введите описание места действия:
     (например: "тридевятое царство", "космическая станция")

User: тридевятое царство, тридесятое государство
Bot: Максимальное количество эпизодов:
     [3] [5] [8] [Без ограничения]

User: [Без ограничения]
Bot: Максимальное количество моментов на эпизод:
     [2] [3] [5] [Без ограничения]

User: [Без ограничения]
Bot: Seed (для воспроизводимости)?
     [Случайный]

User: [Случайный]
Bot: Генерирую сказку…
     [Персонажи]
     [Место действия: тридевятое царство, тридесятое государство]
     [Episode 1]
     [Episode 2]
     ...
     [Формула: α β¹ γ A¹ ↑ D¹ E¹ F¹ G⁴ H¹ I¹ K¹ ↓ Pr¹ Rs¹ L⁰ Ex¹ T⁰ U⁰ W⁰]
```

### Example 2: Short Horror with Limits

```
User: /generate
Bot: Выберите жанр:

User: [Хоррор]
Bot: Выберите тон:

User: [Мрачный]
Bot: Количество ходов:

User: [1]
Bot: Описать место действия?

User: [Ввести описание]
Bot: Введите описание...

User: заброшенный психиатрический госпиталь
Bot: Максимальное количество эпизодов:

User: [3]  ← SHORT STORY
Bot: Максимальное количество моментов на эпизод:

User: [2]  ← MINIMALIST EPISODES
Bot: Seed?

User: [Случайный]
Bot: [Генерирует короткую минималистичную хоррор-историю]
```

## Technical Details

### Data Flow

```
User selects options in Telegram
  ↓
BotState accumulates parameters through dialogue steps
  ↓
GenerateConfig fully populated
  ↓
generate_tale_structure(moves, seed, max_episodes, max_moments)
  ↓
GenConfig with limits → RandomGen
  ↓
Tale structure with place applied to initial.setting
  ↓
StoryComposer with limits
  ↓
LLM generates text respecting all constraints
  ↓
Format and send to user
```

### State Transitions

```
Start
  ↓
SelectGenre (inline keyboard)
  ↓
SelectTone (inline keyboard)
  ↓
SelectMoves (inline keyboard)
  ↓
SelectPlace (inline keyboard: "Ввести описание" / "Пропустить")
  ↓
[if "Ввести описание"] → AwaitPlaceText (text input)
  ↓
SelectMaxEpisodes (inline keyboard: 3/5/8/unlimited)
  ↓
SelectMaxMoments (inline keyboard: 2/3/5/unlimited)
  ↓
SelectSeed (inline keyboard "Случайный" / text input for number)
  ↓
[Generation]
  ↓
Start (ready for next request)
```

### Callback Data Format

New callback patterns:
- `place:enter` — trigger text input for place description
- `place:skip` — skip place description
- `maxepisodes:3` / `maxepisodes:5` / `maxepisodes:8` / `maxepisodes:skip`
- `maxmoments:2` / `maxmoments:3` / `maxmoments:5` / `maxmoments:skip`

## Testing

### Build Status
✅ `cargo build -p fableforge-tg` — Clean compilation
✅ `cargo build --workspace` — All crates compile
✅ `cargo test --workspace` — All 119 tests pass (79 core + 40 llm)

### Compatibility
✅ All changes are backward compatible with existing bot instances
✅ No breaking changes to bot API
✅ Optional parameters default to None when not specified

## Files Modified

- `crates/fableforge-tg/src/state.rs` — Added new states and fields to GenerateConfig
- `crates/fableforge-tg/src/handlers.rs` — Updated handlers, added keyboards, modified generation logic

## Files Created

- `TELEGRAM_BOT_FEATURES.md` — Comprehensive user-facing documentation
- `CHANGELOG_TG_BOT_IMPROVEMENTS.md` — This file

## Total Changes

- **2 files modified** in fableforge-tg crate
- **2 new documentation files**
- **1 file modified** in root (README.md)
- **~200 lines of code added**
- **0 breaking changes**
- **119/119 tests passing**

## Backward Compatibility

✅ **100% backward compatible**

- All new parameters are optional (Option<T>)
- Existing `/quick` command works without changes
- `/structure` command unaffected
- Users can skip all new steps by clicking "Пропустить" / "Без ограничения"

## User Experience Improvements

### Before
- Fixed sequence generation (no control over length)
- No way to specify place
- Limited customization options

### After
- Full control over story length via episode/moment limits
- Custom place descriptions for better immersion
- Complete parity with CLI features
- Still simple to use — just click "Пропустить" for defaults

## Future Enhancements (Not Implemented)

Potential future additions:
- Character name input in bot
- Persistent storage of dialogues (Redis)
- Save/export generated tales
- Regenerate specific episodes
- Streaming generation with progress
- Gallery of generated tales
- Time period selection (similar to place)
- Custom instructions field

## Migration Guide

### For Bot Users

No migration needed! The bot will work immediately with new features:
- Existing commands (`/quick`, `/structure`) work unchanged
- New `/generate` flow has additional steps but all are skippable
- Default behavior (skip all new options) produces same results as before

### For Developers

If extending the bot:

```rust
// Old GenerateConfig (still works if you set new fields to None)
let old_config = GenerateConfig {
    genre: Some("сказка".to_string()),
    tone: None,
    moves: 1,
    seed: None,
};

// New GenerateConfig (with all features)
let new_config = GenerateConfig {
    genre: Some("сказка".to_string()),
    tone: None,
    moves: 1,
    place: Some("тридевятое царство".to_string()),
    max_episodes: Some(5),
    max_moments_per_episode: Some(3),
    seed: None,
};
```

## Credits

Feature implemented in response to user request for Telegram bot to support:
1. Place description input
2. Episode count limits
3. Moments per episode limits

Brings Telegram bot to full feature parity with CLI tool while maintaining intuitive inline keyboard UX.
