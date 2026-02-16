use std::sync::Arc;

use fableforge_core::{Formula, GenConfig, Generator, Lang, RandomGen};
use fableforge_llm::{
    AnyClient, ClaudeClient, OpenRouterClient, StoryComposer, StyleConfig,
};
use teloxide::{
    dispatching::dialogue::InMemStorage,
    payloads::SendMessageSetters,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode},
    utils::command::BotCommands,
};
use tracing::{error, info};

use crate::{
    format,
    state::{BotState, GenerateConfig},
};

type BotDialogue = Dialogue<BotState, InMemStorage<BotState>>;

/// Bot commands.
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Доступные команды:")]
pub enum Command {
    #[command(description = "Приветствие и описание бота")]
    Start,
    #[command(description = "Справка по командам")]
    Help,
    #[command(description = "Генерация сказки (интерактивный диалог)")]
    Generate,
    #[command(description = "Быстрая генерация со случайными параметрами")]
    Quick,
    #[command(description = "Показать морфологическую структуру (без LLM)")]
    Structure(String),
}

/// LLM provider configuration.
#[derive(Clone)]
pub enum LlmProvider {
    Claude { api_key: String, model: String },
    OpenRouter { api_key: String, model: String },
}

impl LlmProvider {
    fn build_client(&self) -> AnyClient {
        match self {
            LlmProvider::Claude { api_key, model } => {
                AnyClient::Claude(ClaudeClient::new(api_key).with_model(model))
            }
            LlmProvider::OpenRouter { api_key, model } => {
                AnyClient::OpenRouter(
                    OpenRouterClient::new(api_key)
                        .with_model(model)
                        .with_app_name("FableForge"),
                )
            }
        }
    }
}

/// Shared dependencies injected into handlers.
#[derive(Clone)]
pub struct Deps {
    pub llm_provider: Option<LlmProvider>,
}

/// Set up the dispatcher and run the bot.
pub async fn run(bot: Bot, deps: Arc<Deps>) {
    let handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<BotState>, BotState>()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(handle_command),
        )
        .branch(dptree::entry().endpoint(handle_text_input));

    let callback_handler = Update::filter_callback_query()
        .enter_dialogue::<CallbackQuery, InMemStorage<BotState>, BotState>()
        .endpoint(handle_callback);

    Dispatcher::builder(
        bot,
        dptree::entry().branch(handler).branch(callback_handler),
    )
    .dependencies(dptree::deps![
        InMemStorage::<BotState>::new(),
        deps
    ])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn handle_command(
    bot: Bot,
    dialogue: BotDialogue,
    msg: Message,
    cmd: Command,
    deps: Arc<Deps>,
) -> anyhow::Result<()> {
    match cmd {
        Command::Start => {
            bot.send_message(
                msg.chat.id,
                "Добро пожаловать в FableForge!\n\n\
                 Я генерирую сказки на основе морфологии Проппа.\n\n\
                 /generate \u{2014} интерактивная генерация сказки\n\
                 /quick \u{2014} быстрая генерация\n\
                 /structure \u{2014} показать морфологическую структуру\n\
                 /help \u{2014} справка",
            )
            .await?;
            dialogue.update(BotState::Start).await?;
        }
        Command::Help => {
            bot.send_message(
                msg.chat.id,
                Command::descriptions().to_string(),
            )
            .await?;
        }
        Command::Generate => {
            if deps.llm_provider.is_none() {
                bot.send_message(
                    msg.chat.id,
                    "API-ключ не настроен. Доступна только /structure.",
                )
                .await?;
                return Ok(());
            }
            let keyboard = genre_keyboard();
            bot.send_message(msg.chat.id, "Выберите жанр:")
                .reply_markup(keyboard)
                .await?;
            dialogue.update(BotState::SelectGenre).await?;
        }
        Command::Quick => {
            do_generate(
                bot,
                msg.chat.id,
                &deps,
                &GenerateConfig {
                    genre: None,
                    tone: None,
                    moves: 1,
                    place: None,
                    max_episodes: None,
                    max_moments_per_episode: None,
                    seed: None,
                },
            )
            .await?;
            dialogue.update(BotState::Start).await?;
        }
        Command::Structure(args) => {
            let moves: usize = args.trim().parse().unwrap_or(1);
            do_structure(bot, msg.chat.id, moves, None).await?;
        }
    }
    Ok(())
}

async fn handle_text_input(
    bot: Bot,
    dialogue: BotDialogue,
    msg: Message,
    deps: Arc<Deps>,
) -> anyhow::Result<()> {
    let state = dialogue.get().await?.unwrap_or_default();
    let text = msg.text().unwrap_or("");

    match state {
        BotState::AwaitPlaceText {
            genre,
            tone,
            moves,
        } => {
            let place = if text.trim().is_empty() {
                None
            } else {
                Some(text.trim().to_string())
            };

            let keyboard = max_episodes_keyboard();
            bot.send_message(
                msg.chat.id,
                "Максимальное количество эпизодов:",
            )
            .reply_markup(keyboard)
            .await?;

            dialogue
                .update(BotState::SelectMaxEpisodes {
                    genre,
                    tone,
                    moves,
                    place,
                })
                .await?;
        }
        BotState::SelectSeed {
            genre,
            tone,
            moves,
            place,
            max_episodes,
            max_moments_per_episode,
        } => {
            let seed = text.trim().parse::<u64>().ok();
            let config = GenerateConfig {
                genre,
                tone,
                moves,
                place,
                max_episodes,
                max_moments_per_episode,
                seed,
            };
            dialogue.update(BotState::Start).await?;
            do_generate(bot, msg.chat.id, &deps, &config).await?;
        }
        _ => {
            bot.send_message(
                msg.chat.id,
                "Используйте команды: /generate, /quick, /structure, /help",
            )
            .await?;
        }
    }
    Ok(())
}

async fn handle_callback(
    bot: Bot,
    dialogue: BotDialogue,
    q: CallbackQuery,
    deps: Arc<Deps>,
) -> anyhow::Result<()> {
    bot.answer_callback_query(&q.id).await?;

    let data = q.data.unwrap_or_default();
    let chat_id = q.message.as_ref().map(|m| m.chat().id).unwrap_or(ChatId(0));
    let state = dialogue.get().await?.unwrap_or_default();

    match state {
        BotState::SelectGenre => {
            let genre = parse_callback_value(&data, "genre");
            let keyboard = tone_keyboard();
            bot.send_message(chat_id, "Выберите тон повествования:")
                .reply_markup(keyboard)
                .await?;
            dialogue.update(BotState::SelectTone { genre }).await?;
        }
        BotState::SelectTone { genre } => {
            let tone = parse_callback_value(&data, "tone");
            let keyboard = moves_keyboard();
            bot.send_message(
                chat_id,
                "Количество ходов (сюжетных арок):",
            )
            .reply_markup(keyboard)
            .await?;
            dialogue
                .update(BotState::SelectMoves { genre, tone })
                .await?;
        }
        BotState::SelectMoves { genre, tone } => {
            let moves: usize = data
                .strip_prefix("moves:")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let keyboard = place_keyboard();
            bot.send_message(
                chat_id,
                "Описать место действия?\n\
                 Нажмите \"Пропустить\" или кнопку \"Ввести описание\":",
            )
            .reply_markup(keyboard)
            .await?;
            dialogue
                .update(BotState::SelectPlace { genre, tone, moves })
                .await?;
        }
        BotState::SelectPlace { genre, tone, moves } => {
            if data == "place:skip" {
                let keyboard = max_episodes_keyboard();
                bot.send_message(
                    chat_id,
                    "Максимальное количество эпизодов:",
                )
                .reply_markup(keyboard)
                .await?;
                dialogue
                    .update(BotState::SelectMaxEpisodes {
                        genre,
                        tone,
                        moves,
                        place: None,
                    })
                    .await?;
            } else if data == "place:enter" {
                bot.send_message(
                    chat_id,
                    "Введите описание места действия:\n\
                     (например: \"тридевятое царство\", \"космическая станция\", \"туманный Лондон XIX века\")",
                )
                .await?;
                dialogue
                    .update(BotState::AwaitPlaceText { genre, tone, moves })
                    .await?;
            }
        }
        BotState::SelectMaxEpisodes {
            genre,
            tone,
            moves,
            place,
        } => {
            let max_episodes = parse_callback_value(&data, "maxepisodes");
            let keyboard = max_moments_keyboard();
            bot.send_message(
                chat_id,
                "Максимальное количество моментов на эпизод:",
            )
            .reply_markup(keyboard)
            .await?;
            dialogue
                .update(BotState::SelectMaxMoments {
                    genre,
                    tone,
                    moves,
                    place,
                    max_episodes: max_episodes.and_then(|v| v.parse().ok()),
                })
                .await?;
        }
        BotState::SelectMaxMoments {
            genre,
            tone,
            moves,
            place,
            max_episodes,
        } => {
            let max_moments_per_episode = parse_callback_value(&data, "maxmoments");
            let keyboard = seed_keyboard();
            bot.send_message(
                chat_id,
                "Seed (для воспроизводимости)?\n\
                 Нажмите \"Случайный\" или введите число:",
            )
            .reply_markup(keyboard)
            .await?;
            dialogue
                .update(BotState::SelectSeed {
                    genre,
                    tone,
                    moves,
                    place,
                    max_episodes,
                    max_moments_per_episode: max_moments_per_episode.and_then(|v| v.parse().ok()),
                })
                .await?;
        }
        BotState::SelectSeed {
            genre,
            tone,
            moves,
            place,
            max_episodes,
            max_moments_per_episode,
        } => {
            let seed = if data == "seed:skip" {
                None
            } else {
                data.strip_prefix("seed:").and_then(|v| v.parse().ok())
            };
            let config = GenerateConfig {
                genre,
                tone,
                moves,
                place,
                max_episodes,
                max_moments_per_episode,
                seed,
            };
            dialogue.update(BotState::Start).await?;
            do_generate(bot, chat_id, &deps, &config).await?;
        }
        _ => {}
    }
    Ok(())
}

fn parse_callback_value(data: &str, prefix: &str) -> Option<String> {
    let val = data.strip_prefix(&format!("{}:", prefix))?;
    if val == "skip" {
        None
    } else {
        Some(val.to_string())
    }
}

// ── Keyboards ──────────────────────────────────────────────

fn genre_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Сказка", "genre:сказка"),
            InlineKeyboardButton::callback("Фэнтези", "genre:фэнтези"),
        ],
        vec![
            InlineKeyboardButton::callback("Нуар", "genre:нуар"),
            InlineKeyboardButton::callback("Хоррор", "genre:хоррор"),
        ],
        vec![
            InlineKeyboardButton::callback("Ироничная", "genre:ироничная"),
            InlineKeyboardButton::callback("Пропустить", "genre:skip"),
        ],
    ])
}

fn tone_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("Лирический", "tone:лирический"),
            InlineKeyboardButton::callback("Мрачный", "tone:мрачный"),
        ],
        vec![
            InlineKeyboardButton::callback("Ироничный", "tone:ироничный"),
            InlineKeyboardButton::callback("Эпический", "tone:эпический"),
        ],
        vec![InlineKeyboardButton::callback("Пропустить", "tone:skip")],
    ])
}

fn moves_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("1", "moves:1"),
        InlineKeyboardButton::callback("2", "moves:2"),
        InlineKeyboardButton::callback("3", "moves:3"),
    ]])
}

fn place_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Ввести описание", "place:enter"),
        InlineKeyboardButton::callback("Пропустить", "place:skip"),
    ]])
}

fn max_episodes_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("3", "maxepisodes:3"),
            InlineKeyboardButton::callback("5", "maxepisodes:5"),
            InlineKeyboardButton::callback("8", "maxepisodes:8"),
        ],
        vec![InlineKeyboardButton::callback("Без ограничения", "maxepisodes:skip")],
    ])
}

fn max_moments_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("2", "maxmoments:2"),
            InlineKeyboardButton::callback("3", "maxmoments:3"),
            InlineKeyboardButton::callback("5", "maxmoments:5"),
        ],
        vec![InlineKeyboardButton::callback("Без ограничения", "maxmoments:skip")],
    ])
}

fn seed_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![vec![
        InlineKeyboardButton::callback("Случайный", "seed:skip"),
    ]])
}

// ── Generation logic ───────────────────────────────────────

async fn do_generate(
    bot: Bot,
    chat_id: ChatId,
    deps: &Deps,
    config: &GenerateConfig,
) -> anyhow::Result<()> {
    let provider = match deps.llm_provider {
        Some(ref p) => p,
        None => {
            bot.send_message(
                chat_id,
                "API-ключ не настроен. Доступна только /structure.",
            )
            .await?;
            return Ok(());
        }
    };

    bot.send_message(chat_id, "Генерирую сказку\u{2026}")
        .await?;

    // Generate morphological structure
    let mut tale = match generate_tale_structure(config.moves, config.seed, config.max_episodes, config.max_moments_per_episode) {
        Ok(tale) => tale,
        Err(e) => {
            error!("Structure generation failed: {}", e);
            bot.send_message(
                chat_id,
                format!("Ошибка генерации структуры: {}", e),
            )
            .await?;
            return Ok(());
        }
    };

    // Apply place to initial situation if provided
    if let Some(ref place) = config.place {
        let mut initial = tale.initial.take().unwrap_or_default();
        let mut setting = initial.setting.take().unwrap_or_default();
        setting.place = Some(place.clone());
        initial.setting = Some(setting);
        tale.initial = Some(initial);
    }

    let formula = Formula::from_tale(&tale);
    info!(
        "Generated structure: {} moves, formula: {}",
        tale.moves.len(),
        formula.to_text()
    );

    // Build style config
    let mut style = StyleConfig::new();
    if let Some(ref g) = config.genre {
        style = style.genre(g);
    }
    if let Some(ref t) = config.tone {
        style = style.tone(t);
    }

    // Generate story text via LLM
    let client = provider.build_client();
    let mut composer = StoryComposer::new(client, style).with_lang(Lang::Ru);

    // Apply limits
    if let Some(max_eps) = config.max_episodes {
        composer = composer.with_max_episodes(max_eps);
    }
    if let Some(max_moments) = config.max_moments_per_episode {
        composer = composer.with_max_moments_per_episode(max_moments);
    }

    let story = match composer.compose(&tale).await {
        Ok(s) => s,
        Err(e) => {
            error!("LLM generation failed: {}", e);
            bot.send_message(
                chat_id,
                format!("Ошибка генерации текста: {}", e),
            )
            .await?;
            return Ok(());
        }
    };

    // Send story parts
    let parts = format::format_story(&story);
    for part in &parts {
        bot.send_message(chat_id, part)
            .parse_mode(ParseMode::Html)
            .await?;
    }

    // Send formula
    bot.send_message(
        chat_id,
        format!(
            "\n<b>Формула:</b> {}",
            format::escape_html_pub(&formula.to_text())
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

async fn do_structure(
    bot: Bot,
    chat_id: ChatId,
    moves: usize,
    seed: Option<u64>,
) -> anyhow::Result<()> {
    let tale = match generate_tale_structure(moves, seed, None, None) {
        Ok(t) => t,
        Err(e) => {
            bot.send_message(chat_id, format!("Ошибка: {}", e)).await?;
            return Ok(());
        }
    };

    let text = format::format_structure(&tale, Lang::Ru);

    // Split if too long
    if text.len() <= 4096 {
        bot.send_message(chat_id, &text)
            .parse_mode(ParseMode::Html)
            .await?;
    } else {
        let mut chunk = String::new();
        for line in text.lines() {
            if chunk.len() + line.len() + 1 > 4096 {
                bot.send_message(chat_id, &chunk)
                    .parse_mode(ParseMode::Html)
                    .await?;
                chunk.clear();
            }
            chunk.push_str(line);
            chunk.push('\n');
        }
        if !chunk.is_empty() {
            bot.send_message(chat_id, &chunk)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    // Formula
    let formula = Formula::from_tale(&tale);
    bot.send_message(
        chat_id,
        format!(
            "<b>Формула:</b> {}",
            format::escape_html_pub(&formula.to_text())
        ),
    )
    .parse_mode(ParseMode::Html)
    .await?;

    Ok(())
}

fn generate_tale_structure(
    moves: usize,
    seed: Option<u64>,
    max_episodes: Option<usize>,
    max_moments_per_episode: Option<usize>,
) -> Result<fableforge_core::Tale, String> {
    let mut gen_config = GenConfig::new()
        .with_move_count(moves..moves + 1)
        .with_max_absurdity(0.5);

    if let Some(s) = seed {
        gen_config = gen_config.with_seed(s);
    }

    // Apply structural limits if provided
    if let Some(max_eps) = max_episodes {
        gen_config = gen_config.with_max_episodes(max_eps);
    }
    if let Some(max_moments) = max_moments_per_episode {
        gen_config = gen_config.with_max_moments_per_episode(max_moments);
    }

    let mut generator = RandomGen::new();

    // Try up to 5 times with different seeds if absurdity is too high
    for attempt in 0..5 {
        match generator.generate(&gen_config) {
            Ok(tale) => return Ok(tale),
            Err(e) => {
                if attempt == 4 {
                    return Err(format!("{}", e));
                }
                gen_config.seed = None;
                generator = RandomGen::new();
            }
        }
    }

    Err("Не удалось сгенерировать структуру".to_string())
}
