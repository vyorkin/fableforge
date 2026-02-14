use fableforge_core::{Connective, Lang, Phase, Tale};
use fableforge_llm::GeneratedStory;

/// Maximum Telegram message length.
const MAX_MESSAGE_LEN: usize = 4096;

/// Format a generated story into Telegram-safe message chunks.
pub fn format_story(story: &GeneratedStory) -> Vec<String> {
    let mut parts = Vec::new();

    // First message: characters + setting
    let mut header = String::from("<b>ПЕРСОНАЖИ:</b>\n");
    for ch in &story.characters {
        let epithet = ch.epithet.as_deref().unwrap_or("");
        header.push_str(&format!(
            "  \u{2022} {} ({})\n",
            escape_html(&ch.name),
            escape_html(epithet)
        ));
        if let Some(ref appearance) = ch.appearance {
            header.push_str(&format!("    {}\n", escape_html(appearance)));
        }
    }
    header.push_str(&format!(
        "\n<b>МЕСТО ДЕЙСТВИЯ:</b>\n  {}\n",
        escape_html(&story.setting)
    ));
    parts.push(header);

    // Episodes as separate messages
    for ep in &story.episodes {
        split_text(&ep.text, &mut parts);
    }

    parts
}

/// Format morphological structure for Telegram (no LLM).
pub fn format_structure(tale: &Tale, lang: Lang) -> String {
    let mut out = String::new();

    // Initial situation
    if let Some(ref initial) = tale.initial {
        out.push_str("<b>Исходная ситуация:</b>\n");
        if let Some(ref setting) = initial.setting {
            if let Some(ref time) = setting.time {
                out.push_str(&format!("  Время: {}\n", escape_html(time)));
            }
            if let Some(ref place) = setting.place {
                out.push_str(&format!("  Место: {}\n", escape_html(place)));
            }
        }
        if let Some(ref context) = initial.context {
            out.push_str(&format!("  Контекст: {}\n", escape_html(context)));
        }
        out.push('\n');
    }

    // Characters
    out.push_str("<b>Персонажи:</b>\n");
    for persona in &tale.personae {
        let spheres: Vec<_> = persona.spheres.iter().map(|s| s.name(lang)).collect();
        out.push_str(&format!(
            "  [{}] {}\n",
            persona.id.0,
            escape_html(&spheres.join(", "))
        ));
    }

    // Moves
    out.push_str("\n<b>Ходы:</b>\n");
    for (i, mov) in tale.moves.iter().enumerate() {
        out.push_str(&format!("\n  <b>Ход {}:</b>\n", i + 1));

        let mut current_phase: Option<Phase> = None;
        for moment in &mov.moments {
            let phase = moment.function.function.phase();
            if current_phase != Some(phase) {
                current_phase = Some(phase);
                if phase == Phase::Donor && mov.triplication {
                    out.push_str(&format!(
                        "    [{}] \u{00d7}3 [УТРОЕНИЕ]\n",
                        escape_html(phase.name(lang))
                    ));
                } else {
                    out.push_str(&format!("    [{}]\n", escape_html(phase.name(lang))));
                }
            }

            let symbol = moment.function.to_notation();
            let desc = moment.function.full_description(lang);
            out.push_str(&format!(
                "      {} \u{2014} {}",
                escape_html(&symbol),
                escape_html(&desc)
            ));

            if let Some(agent) = moment.agent {
                out.push_str(&format!(" (агент: {})", agent.0));
            }
            if let Some(patient) = moment.patient {
                out.push_str(&format!(" (пациент: {})", patient.0));
            }
            out.push('\n');

            if let Some(ref connective) = moment.connective {
                let (label, text) = format_connective(connective);
                out.push_str(&format!(
                    "        \u{21b3} {}: {}\n",
                    label,
                    escape_html(text)
                ));
            }
        }

        // Embedded moves
        for (j, emov) in mov.embedded_moves.iter().enumerate() {
            out.push_str(&format!("\n    <b>Вложенный ход {}:</b>\n", j + 1));
            let mut current_phase: Option<Phase> = None;
            for moment in &emov.moments {
                let phase = moment.function.function.phase();
                if current_phase != Some(phase) {
                    current_phase = Some(phase);
                    out.push_str(&format!(
                        "        [{}]\n",
                        escape_html(phase.name(lang))
                    ));
                }
                let symbol = moment.function.to_notation();
                let desc = moment.function.full_description(lang);
                out.push_str(&format!(
                    "          {} \u{2014} {}",
                    escape_html(&symbol),
                    escape_html(&desc)
                ));
                if let Some(agent) = moment.agent {
                    out.push_str(&format!(" (агент: {})", agent.0));
                }
                if let Some(patient) = moment.patient {
                    out.push_str(&format!(" (пациент: {})", patient.0));
                }
                out.push('\n');
            }
        }
    }

    out
}

fn format_connective(connective: &Connective) -> (&'static str, &str) {
    match connective {
        Connective::Motivation(text) => ("Мотивация", text.as_str()),
        Connective::Transference(text) => ("Перемещение", text.as_str()),
        Connective::Temporal(text) => ("Время", text.as_str()),
        Connective::Custom(text) => ("Указание", text.as_str()),
    }
}

/// Split text into chunks that fit within Telegram's message limit.
fn split_text(text: &str, out: &mut Vec<String>) {
    if text.len() <= MAX_MESSAGE_LEN {
        out.push(escape_html(text));
        return;
    }

    let mut remaining = text;
    while !remaining.is_empty() {
        if remaining.len() <= MAX_MESSAGE_LEN {
            out.push(escape_html(remaining));
            break;
        }

        // Find a paragraph break within the limit
        let chunk = &remaining[..MAX_MESSAGE_LEN];
        let split_at = chunk
            .rfind("\n\n")
            .or_else(|| chunk.rfind('\n'))
            .or_else(|| chunk.rfind(". "))
            .unwrap_or(MAX_MESSAGE_LEN);

        let split_at = split_at.max(1);
        out.push(escape_html(&remaining[..split_at]));
        remaining = remaining[split_at..].trim_start();
    }
}

/// Escape HTML special characters for Telegram HTML parse mode (public).
pub fn escape_html_pub(s: &str) -> String {
    escape_html(s)
}

/// Escape HTML special characters for Telegram HTML parse mode.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
