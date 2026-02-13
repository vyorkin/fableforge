//! FableForge CLI — генератор сказок на основе морфологии Проппа.

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::fmt::format::FmtSpan;

use fableforge_core::{GenConfig, Generator, RandomGen};
use fableforge_llm::{ClaudeClient, StoryComposer, StyleConfig};

#[derive(Parser)]
#[command(name = "fableforge")]
#[command(about = "Генератор сказок на основе морфологии Проппа", long_about = None)]
struct Cli {
    /// Уровень логирования (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: Level,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Сгенерировать сказку
    Generate(GenerateArgs),

    /// Показать морфологическую структуру сказки
    Structure {
        /// Количество ходов
        #[arg(short, long, default_value = "1")]
        moves: usize,

        /// Seed для воспроизводимой генерации
        #[arg(long)]
        seed: Option<u64>,
    },
}

#[derive(Args)]
struct GenerateArgs {
    /// Жанр (детектив, триллер, драма, фэнтези, хоррор и т.д.)
    #[arg(short, long)]
    genre: Option<String>,

    /// Сеттинг (современный город, средневековье, космос и т.д.)
    #[arg(short, long)]
    setting: Option<String>,

    /// Тон повествования (мрачный, ироничный, лиричный и т.д.)
    #[arg(short, long)]
    tone: Option<String>,

    /// Эпоха (современность, XIX век, будущее и т.д.)
    #[arg(short, long)]
    era: Option<String>,

    /// Дополнительные указания по стилю
    #[arg(short, long)]
    custom: Option<String>,

    /// Количество ходов (сюжетных арок)
    #[arg(short, long, default_value = "1")]
    moves: usize,

    /// Seed для воспроизводимой генерации структуры
    #[arg(long)]
    seed: Option<u64>,

    /// Модель Claude (claude-sonnet-4-20250514, claude-opus-4-20250514 и т.д.)
    #[arg(short = 'M', long, default_value = "claude-sonnet-4-20250514")]
    model: String,

    /// API ключ Anthropic (или переменная окружения ANTHROPIC_API_KEY)
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    api_key: Option<String>,

    /// Показать только структуру сказки без генерации текста
    #[arg(long)]
    structure_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Настройка логирования
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Generate(args) => {
            generate_tale(args).await?;
        }
        Commands::Structure { moves, seed } => {
            show_structure(moves, seed)?;
        }
    }

    Ok(())
}

async fn generate_tale(args: GenerateArgs) -> Result<()> {
    let GenerateArgs {
        genre,
        setting,
        tone,
        era,
        custom,
        moves,
        seed,
        model,
        api_key,
        structure_only,
    } = args;
    // Генерация морфологической структуры
    info!("Генерация морфологической структуры...");

    let mut gen_config = GenConfig::new()
        .with_move_count(moves..moves + 1)
        .with_max_absurdity(0.5);

    if let Some(s) = seed {
        gen_config = gen_config.with_seed(s);
    }

    let mut generator = RandomGen::new();
    let tale = generator
        .generate(&gen_config)
        .context("Не удалось сгенерировать структуру сказки")?;

    info!(
        "Структура: {} ходов, {} персонажей",
        tale.moves.len(),
        tale.personae.len()
    );

    // Показываем структуру
    print_structure(&tale);

    if structure_only {
        return Ok(());
    }

    // Проверка API ключа
    let api_key = api_key.ok_or_else(|| {
        anyhow::anyhow!(
            "API ключ не указан. Используйте --api-key или установите ANTHROPIC_API_KEY"
        )
    })?;

    if api_key.is_empty() {
        bail!("API ключ пустой");
    }

    // Настройка стиля
    let mut style = StyleConfig::new();
    if let Some(g) = genre {
        style = style.genre(g);
    }
    if let Some(s) = setting {
        style = style.setting_hint(s);
    }
    if let Some(t) = tone {
        style = style.tone(t);
    }
    if let Some(e) = era {
        style = style.era(e);
    }
    if let Some(c) = custom {
        style = style.custom_instructions(c);
    }

    // Создание клиента и генерация
    info!("Генерация текста с помощью {}...", model);

    let client = ClaudeClient::new(api_key).with_model(model);
    let composer = StoryComposer::new(client, style);

    let story = composer
        .compose(&tale)
        .await
        .context("Не удалось сгенерировать текст сказки")?;

    // Вывод результата
    println!("\n{}", "=".repeat(60));
    println!("СКАЗКА");
    println!("{}\n", "=".repeat(60));

    // Персонажи
    println!("ПЕРСОНАЖИ:");
    for char in &story.characters {
        let epithet = char.epithet.as_deref().unwrap_or("");
        let appearance = char.appearance.as_deref().unwrap_or("");
        println!("  • {} ({})", char.name, epithet);
        if !appearance.is_empty() {
            println!("    {}", appearance);
        }
    }

    println!("\nМЕСТО ДЕЙСТВИЯ:");
    println!("  {}", story.setting);

    println!("\n{}", "-".repeat(60));
    println!();
    println!("{}", story.text);
    println!();
    println!("{}", "=".repeat(60));

    Ok(())
}

fn show_structure(moves: usize, seed: Option<u64>) -> Result<()> {
    let mut gen_config = GenConfig::new()
        .with_move_count(moves..moves + 1)
        .with_max_absurdity(0.5);

    if let Some(s) = seed {
        gen_config = gen_config.with_seed(s);
    }

    let mut generator = RandomGen::new();
    let tale = generator
        .generate(&gen_config)
        .context("Не удалось сгенерировать структуру")?;

    print_structure(&tale);

    Ok(())
}

fn print_structure(tale: &fableforge_core::Tale) {
    use fableforge_core::Lang;

    println!("\n--- Морфологическая структура ---\n");

    // Начальная ситуация
    if let Some(ref initial) = tale.initial {
        println!("Начальная ситуация:");
        if let Some(ref setting) = initial.setting {
            if let Some(ref time) = setting.time {
                println!("  Время: {}", time);
            }
            if let Some(ref place) = setting.place {
                println!("  Место: {}", place);
            }
        }
        if let Some(ref context) = initial.context {
            println!("  Контекст: {}", context);
        }
        println!();
    }

    // Персонажи
    println!("Персонажи:");
    for persona in &tale.personae {
        let spheres: Vec<_> = persona
            .spheres
            .iter()
            .map(|s| s.name(Lang::Ru))
            .collect();
        println!("  [{}] {}", persona.id.0, spheres.join(", "));
    }

    // Ходы
    println!("\nХоды:");
    for (i, mov) in tale.moves.iter().enumerate() {
        println!("\n  Ход {}:", i + 1);

        // Группируем по фазам
        let mut current_phase = None;
        for moment in &mov.moments {
            let phase = moment.function.function.phase();
            if current_phase != Some(phase) {
                current_phase = Some(phase);
                println!("    [{}]", phase.name(Lang::Ru));
            }

            let symbol = moment.function.to_notation();
            let desc = moment.function.full_description(Lang::Ru);
            print!("      {} — {}", symbol, desc);

            if let Some(agent) = moment.agent {
                print!(" (агент: {})", agent.0);
            }
            if let Some(patient) = moment.patient {
                print!(" (пациент: {})", patient.0);
            }
            println!();
        }
    }

    println!();
}
