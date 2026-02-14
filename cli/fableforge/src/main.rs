//! FableForge CLI — Propp-morphology–based fairy tale generator.

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use fableforge_core::{Connective, Formula, GenConfig, Generator, Lang, RandomGen};
use fableforge_llm::{
    AnyClient, ClaudeClient, CoherenceEvaluator, CoherenceReport, OpenRouterClient, StoryComposer,
    StyleConfig,
};
use tracing::{Level, info};
use tracing_subscriber::fmt::format::FmtSpan;

#[derive(Parser)]
#[command(name = "fableforge")]
#[command(about = "Propp-morphology–based fairy tale generator", long_about = None)]
struct Cli {
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: Level,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a fairy tale
    Generate(GenerateArgs),

    /// Show the morphological structure of a fairy tale
    Structure {
        /// Number of moves
        #[arg(short, long, default_value = "1")]
        moves: usize,

        /// Seed for reproducible generation
        #[arg(long)]
        seed: Option<u64>,

        /// Language (en, ru)
        #[arg(short = 'L', long, default_value = "ru")]
        lang: String,

        /// Build tale from a Propp formula string instead of random generation
        #[arg(long)]
        from_formula: Option<String>,

        /// Output as formula text (Propp notation) instead of full structure
        #[arg(long)]
        export_formula: bool,

        /// Output as LaTeX instead of full structure
        #[arg(long)]
        latex: bool,
    },
}

#[derive(Args)]
struct GenerateArgs {
    /// Genre (detective, thriller, drama, fantasy, horror, etc.)
    #[arg(short, long)]
    genre: Option<String>,

    /// Setting (modern city, medieval, space, etc.)
    #[arg(short, long)]
    setting: Option<String>,

    /// Narrative tone (dark, ironic, lyrical, etc.)
    #[arg(short, long)]
    tone: Option<String>,

    /// Era (modern times, 19th century, future, etc.)
    #[arg(short, long)]
    era: Option<String>,

    /// Additional style instructions
    #[arg(short, long)]
    custom: Option<String>,

    /// Number of moves (plot arcs)
    #[arg(short, long, default_value = "1")]
    moves: usize,

    /// Seed for reproducible structure generation
    #[arg(long)]
    seed: Option<u64>,

    /// LLM provider (claude, openrouter)
    #[arg(short, long, default_value = "claude")]
    provider: String,

    /// Model name (provider-specific; defaults per provider if omitted)
    #[arg(short = 'M', long)]
    model: Option<String>,

    /// API key (overrides provider-specific env var)
    #[arg(long)]
    api_key: Option<String>,

    /// Show only the structure without generating full text
    #[arg(long)]
    structure_only: bool,

    /// Evaluate coherence of the generated story (LLM-as-judge)
    #[arg(long)]
    evaluate: bool,

    /// Language (en, ru)
    #[arg(short = 'L', long, default_value = "ru")]
    lang: String,

    /// Build tale from a Propp formula string instead of random generation
    #[arg(long)]
    from_formula: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Logging setup
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .init();

    match cli.command {
        Commands::Generate(args) => {
            generate_tale(args).await?;
        }
        Commands::Structure {
            moves,
            seed,
            lang,
            from_formula,
            export_formula,
            latex,
        } => {
            let lang = parse_lang(&lang)?;
            show_structure(moves, seed, lang, from_formula, export_formula, latex)?;
        }
    }

    Ok(())
}

fn parse_lang(s: &str) -> Result<Lang> {
    match s.to_lowercase().as_str() {
        "en" => Ok(Lang::En),
        "ru" => Ok(Lang::Ru),
        other => bail!("Unknown language '{}'. Supported: en, ru", other),
    }
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
        provider,
        model,
        api_key,
        structure_only,
        evaluate,
        lang,
        from_formula,
    } = args;

    let lang = parse_lang(&lang)?;

    // Morphological structure generation
    info!("Generating morphological structure...");

    let tale = if let Some(formula_str) = from_formula {
        let formula = Formula::parse(&formula_str)
            .context("Failed to parse formula")?;
        formula.to_tale()
    } else {
        let mut gen_config = GenConfig::new()
            .with_move_count(moves..moves + 1)
            .with_max_absurdity(0.5);

        if let Some(s) = seed {
            gen_config = gen_config.with_seed(s);
        }

        let mut generator = RandomGen::new();
        generator
            .generate(&gen_config)
            .context("Failed to generate fairy-tale structure")?
    };

    info!(
        "Structure: {} moves, {} characters",
        tale.moves.len(),
        tale.personae.len()
    );

    // Print structure
    print_structure(&tale, lang);

    // Print formula
    let formula = Formula::from_tale(&tale);
    println!("Formula: {}", formula.to_text());
    println!();

    if structure_only {
        return Ok(());
    }

    // API key resolution (--api-key overrides env var)
    let env_key = match provider.as_str() {
        "openrouter" => "OPENROUTER_API_KEY",
        _ => "ANTHROPIC_API_KEY",
    };
    let api_key = api_key
        .or_else(|| std::env::var(env_key).ok())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "API key is not provided. Use --api-key or set {}",
                env_key
            )
        })?;

    if api_key.is_empty() {
        bail!("API key is empty");
    }

    // Default model per provider
    let model = model.unwrap_or_else(|| match provider.as_str() {
        "openrouter" => "anthropic/claude-sonnet-4".to_string(),
        _ => "claude-sonnet-4-20250514".to_string(),
    });

    // Style configuration
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

    // LLM client construction
    info!("Generating text with {} (provider: {})...", model, provider);

    let build_client = |api_key: String, model: String| -> AnyClient {
        match provider.as_str() {
            "openrouter" => AnyClient::OpenRouter(
                OpenRouterClient::new(api_key)
                    .with_model(model)
                    .with_app_name("FableForge"),
            ),
            _ => AnyClient::Claude(ClaudeClient::new(api_key).with_model(model)),
        }
    };

    let eval_api_key = if evaluate { Some(api_key.clone()) } else { None };
    let eval_model = if evaluate { Some(model.clone()) } else { None };
    let eval_style = if evaluate { Some(style.clone()) } else { None };

    let client = build_client(api_key, model);
    let composer = StoryComposer::new(client, style).with_lang(lang);

    let story = composer
        .compose(&tale)
        .await
        .context("Failed to generate fairy-tale text")?;

    // Output
    let (title, chars_label, setting_label) = match lang {
        Lang::En => ("FAIRY TALE", "CHARACTERS:", "SETTING:"),
        Lang::Ru => ("СКАЗКА", "ПЕРСОНАЖИ:", "МЕСТО ДЕЙСТВИЯ:"),
    };

    println!("\n{}", "=".repeat(60));
    println!("{}", title);
    println!("{}\n", "=".repeat(60));

    println!("{}", chars_label);
    for char in &story.characters {
        let epithet = char.epithet.as_deref().unwrap_or("");
        let appearance = char.appearance.as_deref().unwrap_or("");
        println!("  • {} ({})", char.name, epithet);
        if !appearance.is_empty() {
            println!("    {}", appearance);
        }
    }

    println!("\n{}", setting_label);
    println!("  {}", story.setting);

    println!("\n{}", "-".repeat(60));
    println!();
    println!("{}", story.text);
    println!();
    println!("{}", "=".repeat(60));

    // Coherence evaluation
    if evaluate {
        info!("Running coherence evaluation...");
        let eval_client = build_client(eval_api_key.unwrap(), eval_model.unwrap());
        let evaluator = CoherenceEvaluator::new(eval_client, eval_style.unwrap())
            .with_lang(lang);

        let report = evaluator
            .evaluate(&tale, &story)
            .await
            .context("Failed to evaluate story coherence")?;

        print_coherence_report(&report, lang);
    }

    Ok(())
}

fn show_structure(
    moves: usize,
    seed: Option<u64>,
    lang: Lang,
    from_formula: Option<String>,
    export_formula: bool,
    latex: bool,
) -> Result<()> {
    let tale = if let Some(formula_str) = from_formula {
        let formula = Formula::parse(&formula_str)
            .context("Failed to parse formula")?;
        formula.to_tale()
    } else {
        let mut gen_config = GenConfig::new()
            .with_move_count(moves..moves + 1)
            .with_max_absurdity(0.5);

        if let Some(s) = seed {
            gen_config = gen_config.with_seed(s);
        }

        let mut generator = RandomGen::new();
        generator
            .generate(&gen_config)
            .context("Failed to generate structure")?
    };

    if export_formula {
        println!("{}", Formula::from_tale(&tale).to_text());
    } else if latex {
        println!("{}", Formula::from_tale(&tale).to_latex());
    } else {
        print_structure(&tale, lang);
    }

    Ok(())
}

fn print_coherence_report(report: &CoherenceReport, lang: Lang) {
    println!("\n{}", "=".repeat(60));

    match lang {
        Lang::En => {
            println!("COHERENCE EVALUATION");
            println!("{}\n", "=".repeat(60));

            let d = &report.dimensions;
            println!("Overall score:          {:.2}", report.score);
            println!();
            println!("Dimensions:");
            println!("  Character consistency: {:.2}", d.character_consistency);
            println!("  Structural fidelity:   {:.2}", d.structural_fidelity);
            println!("  Episode continuity:    {:.2}", d.episode_continuity);
            println!("  Narrative quality:     {:.2}", d.narrative_quality);

            println!("\nSummary: {}", report.summary);

            if !report.episode_notes.is_empty() {
                println!("\nEpisode notes:");
                for note in &report.episode_notes {
                    println!("  [Episode {}] {}", note.episode_index + 1, note.note);
                }
            }
        }
        Lang::Ru => {
            println!("ОЦЕНКА СВЯЗНОСТИ");
            println!("{}\n", "=".repeat(60));

            let d = &report.dimensions;
            println!("Общий балл:                  {:.2}", report.score);
            println!();
            println!("Критерии:");
            println!("  Соответствие персонажей:    {:.2}", d.character_consistency);
            println!("  Верность структуре:         {:.2}", d.structural_fidelity);
            println!("  Связность эпизодов:         {:.2}", d.episode_continuity);
            println!("  Качество повествования:     {:.2}", d.narrative_quality);

            println!("\nРезюме: {}", report.summary);

            if !report.episode_notes.is_empty() {
                println!("\nЗамечания по эпизодам:");
                for note in &report.episode_notes {
                    println!("  [Эпизод {}] {}", note.episode_index + 1, note.note);
                }
            }
        }
    }

    println!();
}

fn format_connective_cli(connective: &Connective, lang: Lang) -> (&'static str, &str) {
    match connective {
        Connective::Motivation(text) => {
            let label = match lang {
                Lang::En => "Motivation",
                Lang::Ru => "Мотивация",
            };
            (label, text.as_str())
        }
        Connective::Transference(text) => {
            let label = match lang {
                Lang::En => "Transfer",
                Lang::Ru => "Перемещение",
            };
            (label, text.as_str())
        }
        Connective::Temporal(text) => {
            let label = match lang {
                Lang::En => "Time",
                Lang::Ru => "Время",
            };
            (label, text.as_str())
        }
        Connective::Custom(text) => {
            let label = match lang {
                Lang::En => "Note",
                Lang::Ru => "Указание",
            };
            (label, text.as_str())
        }
    }
}

fn print_structure(tale: &fableforge_core::Tale, lang: Lang) {
    match lang {
        Lang::En => println!("\n--- Morphological structure ---\n"),
        Lang::Ru => println!("\n--- Морфологическая структура ---\n"),
    }

    // Initial situation
    if let Some(ref initial) = tale.initial {
        let (init_label, time_label, place_label, ctx_label) = match lang {
            Lang::En => ("Initial situation:", "Time", "Place", "Context"),
            Lang::Ru => ("Исходная ситуация:", "Время", "Место", "Контекст"),
        };
        println!("{}", init_label);
        if let Some(ref setting) = initial.setting {
            if let Some(ref time) = setting.time {
                println!("  {}: {}", time_label, time);
            }
            if let Some(ref place) = setting.place {
                println!("  {}: {}", place_label, place);
            }
        }
        if let Some(ref context) = initial.context {
            println!("  {}: {}", ctx_label, context);
        }
        println!();
    }

    // Characters
    match lang {
        Lang::En => println!("Characters:"),
        Lang::Ru => println!("Персонажи:"),
    }
    for persona in &tale.personae {
        let spheres: Vec<_> =
            persona.spheres.iter().map(|s| s.name(lang)).collect();
        println!(
            "  [{}] {}",
            persona.id.0,
            spheres.join(", ")
        );
    }

    // Moves
    let move_label = match lang {
        Lang::En => "Move",
        Lang::Ru => "Ход",
    };
    match lang {
        Lang::En => println!("\nMoves:"),
        Lang::Ru => println!("\nХоды:"),
    }
    for (i, mov) in tale.moves.iter().enumerate() {
        println!("\n  {} {}:", move_label, i + 1);

        let mut current_phase = None;
        for moment in &mov.moments {
            let phase = moment.function.function.phase();
            if current_phase != Some(phase) {
                current_phase = Some(phase);
                if phase == fableforge_core::Phase::Donor && mov.triplication {
                    let trip_label = match lang {
                        Lang::En => "TRIPLICATION",
                        Lang::Ru => "УТРОЕНИЕ",
                    };
                    println!("    [{}] ×3 [{}]", phase.name(lang), trip_label);
                } else {
                    println!("    [{}]", phase.name(lang));
                }
            }

            let symbol = moment.function.to_notation();
            let desc = moment.function.full_description(lang);
            print!("      {} — {}", symbol, desc);

            let (agent_label, patient_label) = match lang {
                Lang::En => ("agent", "patient"),
                Lang::Ru => ("агент", "пациент"),
            };
            if let Some(agent) = moment.agent {
                print!(" ({}: {})", agent_label, agent.0);
            }
            if let Some(patient) = moment.patient {
                print!(" ({}: {})", patient_label, patient.0);
            }
            println!();

            if let Some(ref connective) = moment.connective {
                let (label, text) = format_connective_cli(connective, lang);
                println!("        {} {}: {}", "↳", label, text);
            }
        }

        // Print embedded moves
        for (j, emov) in mov.embedded_moves.iter().enumerate() {
            let embedded_label = match lang {
                Lang::En => format!("    Embedded move {}:", j + 1),
                Lang::Ru => format!("    Вложенный ход {}:", j + 1),
            };
            println!("\n{}", embedded_label);

            let mut current_phase = None;
            for moment in &emov.moments {
                let phase = moment.function.function.phase();
                if current_phase != Some(phase) {
                    current_phase = Some(phase);
                    println!("        [{}]", phase.name(lang));
                }

                let symbol = moment.function.to_notation();
                let desc = moment.function.full_description(lang);
                print!("          {} — {}", symbol, desc);

                let (agent_label, patient_label) = match lang {
                    Lang::En => ("agent", "patient"),
                    Lang::Ru => ("агент", "пациент"),
                };
                if let Some(agent) = moment.agent {
                    print!(" ({}: {})", agent_label, agent.0);
                }
                if let Some(patient) = moment.patient {
                    print!(" ({}: {})", patient_label, patient.0);
                }
                println!();
            }
        }
    }

    println!();
}
