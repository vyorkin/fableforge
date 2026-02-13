//! FableForge CLI — Propp-morphology–based fairy tale generator.

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use fableforge_core::{GenConfig, Generator, RandomGen};
use fableforge_llm::{ClaudeClient, StoryComposer, StyleConfig};
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

    /// Claude model (claude-sonnet-4-20250514, claude-opus-4-20250514, etc.)
    #[arg(short = 'M', long, default_value = "claude-sonnet-4-20250514")]
    model: String,

    /// Anthropic API key (or ANTHROPIC_API_KEY environment variable)
    #[arg(long, env = "ANTHROPIC_API_KEY")]
    api_key: Option<String>,

    /// Show only the structure without generating full text
    #[arg(long)]
    structure_only: bool,
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

    // Morphological structure generation
    info!("Generating morphological structure...");

    let mut gen_config = GenConfig::new()
        .with_move_count(moves..moves + 1)
        .with_max_absurdity(0.5);

    if let Some(s) = seed {
        gen_config = gen_config.with_seed(s);
    }

    let mut generator = RandomGen::new();
    let tale = generator
        .generate(&gen_config)
        .context("Failed to generate fairy-tale structure")?;

    info!(
        "Structure: {} moves, {} characters",
        tale.moves.len(),
        tale.personae.len()
    );

    // Print structure
    print_structure(&tale);

    if structure_only {
        return Ok(());
    }

    // API key validation
    let api_key = api_key.ok_or_else(|| {
        anyhow::anyhow!(
            "API key is not provided. Use --api-key or set ANTHROPIC_API_KEY"
        )
    })?;

    if api_key.is_empty() {
        bail!("API key is empty");
    }

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

    // LLM client and text generation
    info!("Generating text with {}...", model);

    let client = ClaudeClient::new(api_key).with_model(model);
    let composer = StoryComposer::new(client, style);

    let story = composer
        .compose(&tale)
        .await
        .context("Failed to generate fairy-tale text")?;

    // Output
    println!("\n{}", "=".repeat(60));
    println!("FAIRY TALE");
    println!("{}\n", "=".repeat(60));

    // Characters
    println!("CHARACTERS:");
    for char in &story.characters {
        let epithet = char.epithet.as_deref().unwrap_or("");
        let appearance = char.appearance.as_deref().unwrap_or("");
        println!("  • {} ({})", char.name, epithet);
        if !appearance.is_empty() {
            println!("    {}", appearance);
        }
    }

    println!("\nSETTING:");
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
        .context("Failed to generate structure")?;

    print_structure(&tale);

    Ok(())
}

fn print_structure(tale: &fableforge_core::Tale) {
    use fableforge_core::Lang;

    println!("\n--- Morphological structure ---\n");

    // Initial situation
    if let Some(ref initial) = tale.initial {
        println!("Initial situation:");
        if let Some(ref setting) = initial.setting {
            if let Some(ref time) = setting.time {
                println!("  Time: {}", time);
            }
            if let Some(ref place) = setting.place {
                println!("  Place: {}", place);
            }
        }
        if let Some(ref context) = initial.context {
            println!("  Context: {}", context);
        }
        println!();
    }

    // Characters
    println!("Characters:");
    for persona in &tale.personae {
        let spheres: Vec<_> =
            persona.spheres.iter().map(|s| s.name(Lang::Ru)).collect();
        println!(
            "  [{}] {}",
            persona.id.0,
            spheres.join(", ")
        );
    }

    // Moves
    println!("\nMoves:");
    for (i, mov) in tale.moves.iter().enumerate() {
        println!("\n  Move {}:", i + 1);

        // Group by phases
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
                print!(" (agent: {})", agent.0);
            }
            if let Some(patient) = moment.patient {
                print!(" (patient: {})", patient.0);
            }
            println!();
        }
    }

    println!();
}
