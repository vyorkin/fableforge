//! Prompt building for Claude AI.
//!
//! Constructs prompts for character generation and narrative episodes.

use fableforge_core::{InitialSituation, Lang, Sphere, Tale};

use crate::context::TaleContext;
use crate::episode::Episode;

/// Style configuration for story generation.
#[derive(Debug, Clone, Default)]
pub struct StyleConfig {
    /// Genre (detective, thriller, drama, fantasy, horror, etc.)
    pub genre: Option<String>,
    /// Setting hint (modern city, medieval, space, etc.)
    pub setting_hint: Option<String>,
    /// Narrative tone (dark, ironic, lyrical, etc.)
    pub tone: Option<String>,
    /// Era (modern, 19th century, future, etc.)
    pub era: Option<String>,
    /// Custom style instructions.
    pub custom_instructions: Option<String>,
}

impl StyleConfig {
    /// Create a new empty style config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set genre.
    pub fn genre(mut self, genre: impl Into<String>) -> Self {
        self.genre = Some(genre.into());
        self
    }

    /// Set setting hint.
    pub fn setting_hint(mut self, setting: impl Into<String>) -> Self {
        self.setting_hint = Some(setting.into());
        self
    }

    /// Set tone.
    pub fn tone(mut self, tone: impl Into<String>) -> Self {
        self.tone = Some(tone.into());
        self
    }

    /// Set era.
    pub fn era(mut self, era: impl Into<String>) -> Self {
        self.era = Some(era.into());
        self
    }

    /// Set custom instructions.
    pub fn custom_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.custom_instructions = Some(instructions.into());
        self
    }

    /// Format style section for prompts.
    fn format_style(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref genre) = self.genre {
            parts.push(format!("Жанр: {}", genre));
        }
        if let Some(ref setting) = self.setting_hint {
            parts.push(format!("Сеттинг: {}", setting));
        }
        if let Some(ref tone) = self.tone {
            parts.push(format!("Тон: {}", tone));
        }
        if let Some(ref era) = self.era {
            parts.push(format!("Эпоха: {}", era));
        }
        if let Some(ref custom) = self.custom_instructions {
            parts.push(custom.clone());
        }

        parts.join("\n")
    }
}

/// Prompt builder for story generation.
pub struct PromptBuilder {
    lang: Lang,
    style: StyleConfig,
}

impl PromptBuilder {
    /// Create a new prompt builder with style configuration.
    pub fn new(style: StyleConfig) -> Self {
        Self {
            lang: Lang::Ru,
            style,
        }
    }

    /// Set the language for prompts.
    pub fn with_lang(mut self, lang: Lang) -> Self {
        self.lang = lang;
        self
    }

    /// Build prompt for character generation.
    pub fn character_prompt(&self, tale: &Tale) -> String {
        let style_section = self.style.format_style();

        let mut personae_section = String::new();
        for persona in &tale.personae {
            let spheres_desc = persona
                .spheres
                .iter()
                .map(|s| sphere_description(*s, self.lang))
                .collect::<Vec<_>>()
                .join(", ");

            personae_section.push_str(&format!(
                "{}. Роль в истории: {}\n",
                persona.id.0, spheres_desc
            ));
        }

        format!(
            r#"Ты — писатель, создающий историю.

{style_section}

Придумай персонажей:

{personae_section}
Для каждого персонажа придумай:
- Имя (подходящее для выбранного сеттинга)
- Краткую характеристику (2-3 слова)
- Описание внешности (1-2 предложения)

Также придумай место действия, подходящее для этой истории.

Ответь в формате JSON:
{{
  "characters": [
    {{"id": 1, "name": "...", "epithet": "...", "appearance": "..."}},
    ...
  ],
  "setting": "описание места действия"
}}"#
        )
    }

    /// Build prompt for initial situation / exposition.
    pub fn initial_situation_prompt(
        &self,
        initial: &InitialSituation,
        ctx: &TaleContext,
    ) -> String {
        let style_section = self.style.format_style();
        let characters_section = self.format_characters(ctx);
        let setting = ctx.setting.as_deref().unwrap_or("Неизвестное место");

        let mut prompt = format!(
            r#"Ты — писатель, создающий историю.

{style_section}

ПЕРСОНАЖИ:
{characters_section}

МЕСТО ДЕЙСТВИЯ: {setting}

Напиши начало истории — экспозицию.
Представь персонажей, покажи их обычную жизнь до начала событий."#
        );

        // Add setting/context hints if provided in the tale structure
        if let Some(ref setting_info) = initial.setting {
            if let Some(ref place) = setting_info.place {
                prompt.push_str(&format!("\n\nМесто: {}", place));
            }
            if let Some(ref time) = setting_info.time {
                prompt.push_str(&format!("\nВремя: {}", time));
            }
        }
        if let Some(ref context) = initial.context {
            prompt.push_str(&format!("\n\nКонтекст: {}", context));
        }

        prompt.push_str("\n\nДлина: 2-3 абзаца.");

        prompt
    }

    /// Build prompt for a narrative phase episode.
    pub fn phase_prompt(&self, episode: &Episode, ctx: &TaleContext) -> String {
        let style_section = self.style.format_style();
        let characters_section = self.format_characters_short(ctx);
        let summary = ctx.summary();
        let last_text = ctx.last_text().unwrap_or("(нет)");

        let mut moments_section = String::new();
        for moment in &episode.moments {
            let func_desc = moment.function.function.name(self.lang);
            moments_section.push_str(&format!("- {}\n", func_desc));

            if let Some(agent_id) = moment.agent {
                let agent_name = ctx.character_name(agent_id);
                moments_section.push_str(&format!("  Действует: {}\n", agent_name));
            }
            if let Some(patient_id) = moment.patient {
                let patient_name = ctx.character_name(patient_id);
                moments_section.push_str(&format!("  Объект действия: {}\n", patient_name));
            }
        }

        format!(
            r#"Продолжай историю.

{style_section}

ПЕРСОНАЖИ:
{characters_section}

КРАТКОЕ СОДЕРЖАНИЕ:
{summary}

ПРЕДЫДУЩИЙ ФРАГМЕНТ:
{last_text}

---

В этом эпизоде должно произойти:
{moments_section}
Напиши этот фрагмент истории, органично продолжая предыдущий текст.

Длина: 2-4 абзаца."#
        )
    }

    /// Format full character list for prompts.
    fn format_characters(&self, ctx: &TaleContext) -> String {
        ctx.characters
            .values()
            .map(|c| {
                let epithet = c.epithet.as_deref().unwrap_or("");
                let appearance = c.appearance.as_deref().unwrap_or("");
                format!("- {} ({}): {}", c.name, epithet, appearance)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format short character list (name + epithet only).
    fn format_characters_short(&self, ctx: &TaleContext) -> String {
        ctx.characters
            .values()
            .map(|c| {
                let epithet = c.epithet.as_deref().unwrap_or("");
                format!("- {} ({})", c.name, epithet)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Get localized description for a sphere of action.
fn sphere_description(sphere: Sphere, lang: Lang) -> &'static str {
    match lang {
        Lang::En => match sphere {
            Sphere::Hero => "the hero who seeks or suffers",
            Sphere::Villain => "the villain who causes harm",
            Sphere::Donor => "the donor who provides magical aid",
            Sphere::Helper => "the magical helper",
            Sphere::Princess => "the sought-for person (and their father)",
            Sphere::Dispatcher => "the dispatcher who sends the hero",
            Sphere::FalseHero => "the false hero with unfounded claims",
        },
        Lang::Ru => match sphere {
            Sphere::Hero => "герой-искатель или герой-жертва",
            Sphere::Villain => "вредитель, причиняющий ущерб",
            Sphere::Donor => "даритель волшебного средства",
            Sphere::Helper => "волшебный помощник",
            Sphere::Princess => "искомый персонаж (и его отец)",
            Sphere::Dispatcher => "отправитель героя",
            Sphere::FalseHero => "ложный герой с притязаниями",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_config_builder() {
        let style = StyleConfig::new()
            .genre("нуар-детектив")
            .setting_hint("Москва, 1990-е")
            .tone("мрачный");

        assert_eq!(style.genre, Some("нуар-детектив".to_string()));
        assert_eq!(style.setting_hint, Some("Москва, 1990-е".to_string()));
        assert_eq!(style.tone, Some("мрачный".to_string()));
    }

    #[test]
    fn test_format_style() {
        let style = StyleConfig::new().genre("фэнтези").era("средневековье");

        let formatted = style.format_style();
        assert!(formatted.contains("Жанр: фэнтези"));
        assert!(formatted.contains("Эпоха: средневековье"));
    }

    #[test]
    fn test_character_prompt_structure() {
        let style = StyleConfig::new().genre("сказка");
        let builder = PromptBuilder::new(style);

        let mut tale = Tale::default();
        tale.personae.push(fableforge_core::Persona::new(
            1u32,
            vec![Sphere::Hero],
        ));

        let prompt = builder.character_prompt(&tale);

        assert!(prompt.contains("Ты — писатель"));
        assert!(prompt.contains("Жанр: сказка"));
        assert!(prompt.contains("Роль в истории:"));
        assert!(prompt.contains("формате JSON"));
    }
}
