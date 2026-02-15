//! Prompt building for Claude AI.
//!
//! Constructs prompts for character generation and narrative episodes.

use fableforge_core::{
    Connective, InitialSituation, Lang, Persona, Sphere, Tale,
};

use crate::{
    context::{GeneratedStory, TaleContext},
    episode::Episode,
};

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
    pub fn custom_instructions(
        mut self,
        instructions: impl Into<String>,
    ) -> Self {
        self.custom_instructions = Some(instructions.into());
        self
    }

    /// Format style section for prompts.
    fn format_style(&self, lang: Lang) -> String {
        let mut parts = Vec::new();

        if let Some(ref genre) = self.genre {
            let label = match lang {
                Lang::En => "Genre",
                Lang::Ru => "Жанр",
            };
            parts.push(format!("{}: {}", label, genre));
        }
        if let Some(ref setting) = self.setting_hint {
            let label = match lang {
                Lang::En => "Setting",
                Lang::Ru => "Сеттинг",
            };
            parts.push(format!("{}: {}", label, setting));
        }
        if let Some(ref tone) = self.tone {
            let label = match lang {
                Lang::En => "Tone",
                Lang::Ru => "Тон",
            };
            parts.push(format!("{}: {}", label, tone));
        }
        if let Some(ref era) = self.era {
            let label = match lang {
                Lang::En => "Era",
                Lang::Ru => "Эпоха",
            };
            parts.push(format!("{}: {}", label, era));
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
        let style_section = self.style.format_style(self.lang);

        let has_multi_sphere =
            tale.personae.iter().any(|p| p.spheres.len() > 1);

        let mut personae_section = String::new();
        for persona in &tale.personae {
            let spheres_desc = persona
                .spheres
                .iter()
                .map(|s| sphere_description(*s, self.lang))
                .collect::<Vec<_>>()
                .join(", ");

            let role_label = match self.lang {
                Lang::En => "Story role",
                Lang::Ru => "Роль в истории",
            };
            personae_section.push_str(&format!(
                "{}. {}: {}\n",
                persona.id.0, role_label, spheres_desc
            ));
        }

        let duality_instruction = if has_multi_sphere {
            match self.lang {
                Lang::En => {
                    "\nSome characters combine multiple roles. This duality should be reflected in their personality, appearance, and name.\n"
                }
                Lang::Ru => {
                    "\nНекоторые персонажи совмещают несколько ролей. Это должно отражаться в их характере, внешности и имени — чтобы в них чувствовалась двойственность.\n"
                }
            }
        } else {
            ""
        };

        match self.lang {
            Lang::En => format!(
                r#"You are a writer creating a story.

{style_section}

Create characters:

{personae_section}{duality_instruction}
For each character, create:
- A name (appropriate for the chosen setting)
- A brief epithet (2-3 words)
- An appearance description (1-2 sentences)

Also create a setting appropriate for this story.

Reply in JSON format:
{{
  "characters": [
    {{"id": 1, "name": "...", "epithet": "...", "appearance": "..."}},
    ...
  ],
  "setting": "description of the setting"
}}"#
            ),
            Lang::Ru => format!(
                r#"Ты — писатель, создающий историю.

{style_section}

Придумай персонажей:

{personae_section}{duality_instruction}
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
            ),
        }
    }

    /// Build prompt for initial situation / exposition.
    pub fn initial_situation_prompt(
        &self,
        initial: &InitialSituation,
        ctx: &TaleContext,
        personae: &[Persona],
    ) -> String {
        let style_section = self.style.format_style(self.lang);
        let characters_section = self.format_characters(ctx, personae);

        match self.lang {
            Lang::En => {
                let setting = ctx.setting.as_deref().unwrap_or("Unknown place");

                let mut prompt = format!(
                    r#"You are a writer creating a story.

{style_section}

CHARACTERS:
{characters_section}

SETTING: {setting}

Write the beginning of the story — the exposition.
Introduce the characters, show their ordinary life before the events begin."#
                );

                if let Some(ref setting_info) = initial.setting {
                    if let Some(ref place) = setting_info.place {
                        prompt.push_str(&format!("\n\nPlace: {}", place));
                    }
                    if let Some(ref time) = setting_info.time {
                        prompt.push_str(&format!("\nTime: {}", time));
                    }
                }
                if let Some(ref context) = initial.context {
                    prompt.push_str(&format!("\n\nContext: {}", context));
                }

                prompt.push_str("\n\nLength: 2-3 paragraphs.");
                prompt
            }
            Lang::Ru => {
                let setting =
                    ctx.setting.as_deref().unwrap_or("Неизвестное место");

                let mut prompt = format!(
                    r#"Ты — писатель, создающий историю.

{style_section}

ПЕРСОНАЖИ:
{characters_section}

МЕСТО ДЕЙСТВИЯ: {setting}

Напиши начало истории — экспозицию.
Представь персонажей, покажи их обычную жизнь до начала событий."#
                );

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
        }
    }

    /// Build prompt for a narrative phase episode.
    pub fn phase_prompt(
        &self,
        episode: &Episode,
        ctx: &TaleContext,
        personae: &[Persona],
    ) -> String {
        let style_section = self.style.format_style(self.lang);
        let characters_section = self.format_characters_short(ctx, personae);
        let summary = ctx.summary();

        let mut moments_section = String::new();

        // Mark embedded episodes
        if episode.is_embedded {
            let label = match self.lang {
                Lang::Ru => "[ВЛОЖЕННЫЙ ЭПИЗОД — побочный квест]\n",
                Lang::En => "[EMBEDDED EPISODE — side quest]\n",
            };
            moments_section.push_str(label);
        }

        // Triplication instruction for donor phase
        if episode.triplication {
            let instruction = match self.lang {
                Lang::Ru => {
                    "УТРОЕНИЕ: Этот эпизод содержит утроение — повторение действия три раза с нарастанием.\n"
                }
                Lang::En => {
                    "TRIPLICATION: This episode uses triplication — the action repeats three times with escalation.\n"
                }
            };
            moments_section.push_str(instruction);
        }

        for moment in &episode.moments {
            let func_desc = moment.function.full_description(self.lang);
            let symbol = moment.function.to_notation();
            moments_section.push_str(&format!(
                "- {} ({})\n",
                func_desc, symbol
            ));

            let (agent_label, patient_label) = match self.lang {
                Lang::En => ("  Agent", "  Patient"),
                Lang::Ru => ("  Действует", "  Объект действия"),
            };

            if let Some(agent_id) = moment.agent {
                let agent_name = ctx.character_name(agent_id);
                moments_section.push_str(&format!(
                    "{}: {}\n",
                    agent_label, agent_name
                ));
            }
            if let Some(patient_id) = moment.patient {
                let patient_name = ctx.character_name(patient_id);
                moments_section.push_str(&format!(
                    "{}: {}\n",
                    patient_label, patient_name
                ));
            }

            if let Some(ref connective) = moment.connective {
                let (label, text) = format_connective(connective, self.lang);
                moments_section.push_str(&format!("  {}: {}\n", label, text));
            }
        }

        match self.lang {
            Lang::En => {
                let last_text = ctx.last_text().unwrap_or("(none)");
                format!(
                    r#"Continue the story.

{style_section}

CHARACTERS:
{characters_section}

SUMMARY SO FAR:
{summary}

PREVIOUS FRAGMENT:
{last_text}

---

In this episode, the following should happen:
{moments_section}
Write this fragment of the story, seamlessly continuing from the previous text.

Length: 2-4 paragraphs."#
                )
            }
            Lang::Ru => {
                let last_text = ctx.last_text().unwrap_or("(нет)");
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
        }
    }

    /// Build prompt for coherence evaluation (LLM-as-judge).
    pub fn evaluation_prompt(
        &self,
        tale: &Tale,
        story: &GeneratedStory,
    ) -> String {
        let mut structure = String::new();

        // Personae with spheres
        let personae_header = match self.lang {
            Lang::En => "CHARACTERS (morphological structure):\n",
            Lang::Ru => "ПЕРСОНАЖИ (морфологическая структура):\n",
        };
        structure.push_str(personae_header);
        for persona in &tale.personae {
            let spheres: Vec<_> = persona
                .spheres
                .iter()
                .map(|s| sphere_description(*s, self.lang))
                .collect();
            let name = story
                .characters
                .iter()
                .find(|c| c.id == persona.id)
                .map(|c| c.name.as_str())
                .unwrap_or("?");
            structure.push_str(&format!(
                "- {} (id={}) — {}\n",
                name,
                persona.id.0,
                spheres.join(", "),
            ));
        }

        // Moves with functions
        let (moves_header, move_label, agent_label, patient_label) =
            match self.lang {
                Lang::En => ("\nMOVES:\n", "Move", "agent", "patient"),
                Lang::Ru => ("\nХОДЫ:\n", "Ход", "агент", "объект"),
            };
        structure.push_str(moves_header);
        for (i, mov) in tale.moves.iter().enumerate() {
            structure.push_str(&format!("{} {}:\n", move_label, i + 1));
            for moment in &mov.moments {
                let desc = moment.function.full_description(self.lang);
                let symbol = moment.function.to_notation();
                structure.push_str(&format!("  {} — {}", symbol, desc));
                if let Some(agent_id) = moment.agent {
                    let agent_name = story
                        .characters
                        .iter()
                        .find(|c| c.id == agent_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("?");
                    structure.push_str(&format!(
                        " [{}: {}]",
                        agent_label, agent_name
                    ));
                }
                if let Some(patient_id) = moment.patient {
                    let patient_name = story
                        .characters
                        .iter()
                        .find(|c| c.id == patient_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("?");
                    structure.push_str(&format!(
                        " [{}: {}]",
                        patient_label, patient_name
                    ));
                }
                structure.push('\n');
            }
        }

        // Episodes text
        let episode_label = match self.lang {
            Lang::En => "Episode",
            Lang::Ru => "Эпизод",
        };
        let mut episodes_text = String::new();
        for (i, ep) in story.episodes.iter().enumerate() {
            episodes_text.push_str(&format!(
                "--- {} {} ---\n{}\n\n",
                episode_label,
                i + 1,
                ep.text
            ));
        }

        match self.lang {
            Lang::En => format!(
                r#"You are a literary critic and analyst. Evaluate the quality and coherence of the generated story.

MORPHOLOGICAL STRUCTURE (based on Propp):
{structure}

GENERATED TEXT:
{episodes_text}

Evaluate the story on four criteria (each from 0.0 to 1.0):

1. character_consistency — Character consistency: do characters behave according to their spheres of action (hero, villain, donor, etc.)? Are their characters consistent throughout the text?

2. structural_fidelity — Structural fidelity: are all specified narrative functions realized in the text? Is the phase order maintained?

3. episode_continuity — Episode continuity: does the narrative flow logically from episode to episode? Are there contradictions between fragments?

4. narrative_quality — Narrative quality: style, language, engagement, literary quality of the text.

Reply strictly in JSON format:
{{
  "score": 0.0,
  "dimensions": {{
    "character_consistency": 0.0,
    "structural_fidelity": 0.0,
    "episode_continuity": 0.0,
    "narrative_quality": 0.0
  }},
  "summary": "Brief conclusion about story quality (2-3 sentences).",
  "episode_notes": [
    {{"episode_index": 0, "note": "Note about a specific episode."}}
  ]
}}"#
            ),
            Lang::Ru => format!(
                r#"Ты — литературный критик-аналитик. Оцени качество и связность сгенерированной истории.

МОРФОЛОГИЧЕСКАЯ СТРУКТУРА (по Проппу):
{structure}

СГЕНЕРИРОВАННЫЙ ТЕКСТ:
{episodes_text}

Оцени историю по четырём критериям (каждый от 0.0 до 1.0):

1. character_consistency — Соответствие персонажей: ведут ли себя персонажи согласно своим сферам действия (герой, вредитель, даритель и т.д.)? Последовательны ли их характеры на протяжении всего текста?

2. structural_fidelity — Верность структуре: реализованы ли в тексте все заданные нарративные функции? Соблюдается ли порядок фаз?

3. episode_continuity — Связность эпизодов: логично ли переходит повествование от эпизода к эпизоду? Нет ли противоречий между фрагментами?

4. narrative_quality — Качество повествования: стиль, язык, увлекательность, литературность текста.

Ответь строго в формате JSON:
{{
  "score": 0.0,
  "dimensions": {{
    "character_consistency": 0.0,
    "structural_fidelity": 0.0,
    "episode_continuity": 0.0,
    "narrative_quality": 0.0
  }},
  "summary": "Краткий вывод о качестве истории (2-3 предложения).",
  "episode_notes": [
    {{"episode_index": 0, "note": "Замечание по конкретному эпизоду."}}
  ]
}}"#
            ),
        }
    }

    /// Format full character list for prompts, including sphere roles.
    fn format_characters(
        &self,
        ctx: &TaleContext,
        personae: &[Persona],
    ) -> String {
        ctx.characters
            .values()
            .map(|c| {
                let epithet = c.epithet.as_deref().unwrap_or("");
                let appearance = c.appearance.as_deref().unwrap_or("");
                let roles = self.format_roles(c.id, personae);
                if roles.is_empty() {
                    format!(
                        "- {} ({}): {}",
                        c.name, epithet, appearance
                    )
                } else {
                    format!(
                        "- {} ({}): {} — {}",
                        c.name, epithet, appearance, roles
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format short character list (name + epithet + roles).
    fn format_characters_short(
        &self,
        ctx: &TaleContext,
        personae: &[Persona],
    ) -> String {
        ctx.characters
            .values()
            .map(|c| {
                let epithet = c.epithet.as_deref().unwrap_or("");
                let roles = self.format_roles(c.id, personae);
                if roles.is_empty() {
                    format!("- {} ({})", c.name, epithet)
                } else {
                    format!("- {} ({}) — {}", c.name, epithet, roles)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format sphere roles for a character.
    fn format_roles(
        &self,
        id: fableforge_core::PersonaId,
        personae: &[Persona],
    ) -> String {
        personae
            .iter()
            .find(|p| p.id == id)
            .map(|p| {
                p.spheres
                    .iter()
                    .map(|s| sphere_description(*s, self.lang))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    }
}

/// Format a connective for prompt rendering.
fn format_connective(
    connective: &Connective,
    lang: Lang,
) -> (&'static str, &str) {
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

        assert_eq!(
            style.genre,
            Some("нуар-детектив".to_string())
        );
        assert_eq!(
            style.setting_hint,
            Some("Москва, 1990-е".to_string())
        );
        assert_eq!(style.tone, Some("мрачный".to_string()));
    }

    #[test]
    fn test_format_style_ru() {
        let style = StyleConfig::new().genre("фэнтези").era("средневековье");

        let formatted = style.format_style(Lang::Ru);
        assert!(formatted.contains("Жанр: фэнтези"));
        assert!(formatted.contains("Эпоха: средневековье"));
    }

    #[test]
    fn test_format_style_en() {
        let style = StyleConfig::new().genre("fantasy").era("medieval");

        let formatted = style.format_style(Lang::En);
        assert!(formatted.contains("Genre: fantasy"));
        assert!(formatted.contains("Era: medieval"));
    }

    #[test]
    fn test_character_prompt_structure_ru() {
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

    #[test]
    fn test_character_prompt_structure_en() {
        let style = StyleConfig::new().genre("fairy tale");
        let builder = PromptBuilder::new(style).with_lang(Lang::En);

        let mut tale = Tale::default();
        tale.personae.push(fableforge_core::Persona::new(
            1u32,
            vec![Sphere::Hero],
        ));

        let prompt = builder.character_prompt(&tale);

        assert!(prompt.contains("You are a writer"));
        assert!(prompt.contains("Genre: fairy tale"));
        assert!(prompt.contains("Story role:"));
        assert!(prompt.contains("JSON format"));
    }

    #[test]
    fn test_phase_prompt_with_connective() {
        use fableforge_core::{
            Connective, Moment, NarrativeFunction, NarrativeFunctionInstance,
            Phase,
        };

        use crate::{context::TaleContext, episode::Episode};

        let mut moment = Moment::new(NarrativeFunctionInstance::new(
            NarrativeFunction::Counteraction,
        ));
        moment.connective = Some(Connective::motivation("из мести"));

        let episode = Episode::phase(Phase::Complication, vec![moment]);
        let ctx = TaleContext::new();

        let builder = PromptBuilder::new(StyleConfig::new());
        let prompt = builder.phase_prompt(&episode, &ctx, &[]);

        assert!(
            prompt.contains("Мотивация: из мести"),
            "Expected motivation connective in prompt"
        );
    }

    #[test]
    fn test_phase_prompt_with_connective_en() {
        use fableforge_core::{
            Connective, Moment, NarrativeFunction, NarrativeFunctionInstance,
            Phase,
        };

        use crate::{context::TaleContext, episode::Episode};

        let mut moment = Moment::new(NarrativeFunctionInstance::new(
            NarrativeFunction::Departure,
        ));
        moment.connective = Some(Connective::transference(
            "верхом на коне",
        ));

        let episode = Episode::phase(Phase::Complication, vec![moment]);
        let ctx = TaleContext::new();

        let builder =
            PromptBuilder::new(StyleConfig::new()).with_lang(Lang::En);
        let prompt = builder.phase_prompt(&episode, &ctx, &[]);

        assert!(
            prompt.contains("Transfer: верхом на коне"),
            "Expected transfer connective in English prompt"
        );
    }

    #[test]
    fn test_evaluation_prompt_structure_ru() {
        use fableforge_core::{
            Move, NarrativeFunction, Persona, PersonaId, Phase,
        };

        use crate::{
            context::{EpisodeResult, GeneratedCharacter, GeneratedStory},
            episode::Episode,
        };

        let mut mov = Move::new();
        mov.add_function(NarrativeFunction::Villainy);

        let tale = Tale {
            initial: None,
            personae: vec![
                Persona::new(1u32, vec![Sphere::Hero]),
                Persona::new(2u32, vec![Sphere::Villain]),
            ],
            moves: vec![mov],
        };

        let story = GeneratedStory {
            characters: vec![
                GeneratedCharacter {
                    id: PersonaId(1),
                    name: "Иван".to_string(),
                    epithet: Some("храбрый".to_string()),
                    appearance: None,
                },
                GeneratedCharacter {
                    id: PersonaId(2),
                    name: "Кощей".to_string(),
                    epithet: Some("бессмертный".to_string()),
                    appearance: None,
                },
            ],
            setting: "Тридевятое царство".to_string(),
            text: "Жил-был Иван.".to_string(),
            episodes: vec![EpisodeResult {
                episode: Episode::phase(Phase::Complication, Vec::new()),
                text: "Кощей похитил царевну.".to_string(),
            }],
        };

        let builder = PromptBuilder::new(StyleConfig::new());
        let prompt = builder.evaluation_prompt(&tale, &story);

        assert!(prompt.contains("литературный критик"));
        assert!(prompt.contains("МОРФОЛОГИЧЕСКАЯ СТРУКТУРА"));
        assert!(prompt.contains("Иван"));
        assert!(prompt.contains("Кощей"));
        assert!(prompt.contains("character_consistency"));
        assert!(prompt.contains("structural_fidelity"));
        assert!(prompt.contains("episode_continuity"));
        assert!(prompt.contains("narrative_quality"));
        assert!(prompt.contains("формате JSON"));
        assert!(prompt.contains("Эпизод 1"));
    }

    #[test]
    fn test_evaluation_prompt_structure_en() {
        use fableforge_core::{
            Move, NarrativeFunction, Persona, PersonaId, Phase,
        };

        use crate::{
            context::{EpisodeResult, GeneratedCharacter, GeneratedStory},
            episode::Episode,
        };

        let mut mov = Move::new();
        mov.add_function(NarrativeFunction::Villainy);

        let tale = Tale {
            initial: None,
            personae: vec![
                Persona::new(1u32, vec![Sphere::Hero]),
                Persona::new(2u32, vec![Sphere::Villain]),
            ],
            moves: vec![mov],
        };

        let story = GeneratedStory {
            characters: vec![
                GeneratedCharacter {
                    id: PersonaId(1),
                    name: "Ivan".to_string(),
                    epithet: Some("brave".to_string()),
                    appearance: None,
                },
                GeneratedCharacter {
                    id: PersonaId(2),
                    name: "Koschei".to_string(),
                    epithet: Some("immortal".to_string()),
                    appearance: None,
                },
            ],
            setting: "Far-away kingdom".to_string(),
            text: "Once upon a time there lived Ivan.".to_string(),
            episodes: vec![EpisodeResult {
                episode: Episode::phase(Phase::Complication, Vec::new()),
                text: "Koschei kidnapped the princess.".to_string(),
            }],
        };

        let builder =
            PromptBuilder::new(StyleConfig::new()).with_lang(Lang::En);
        let prompt = builder.evaluation_prompt(&tale, &story);

        assert!(prompt.contains("literary critic"));
        assert!(prompt.contains("MORPHOLOGICAL STRUCTURE"));
        assert!(prompt.contains("Ivan"));
        assert!(prompt.contains("Koschei"));
        assert!(prompt.contains("character_consistency"));
        assert!(prompt.contains("JSON format"));
        assert!(prompt.contains("Episode 1"));
    }

    #[test]
    fn test_phase_prompt_with_triplication() {
        use fableforge_core::{
            Moment, NarrativeFunction, NarrativeFunctionInstance, Phase,
        };

        use crate::{context::TaleContext, episode::Episode};

        let moment = Moment::new(NarrativeFunctionInstance::new(
            NarrativeFunction::DonorTest,
        ));
        let mut episode = Episode::phase(Phase::Donor, vec![moment]);
        episode.triplication = true;

        let ctx = TaleContext::new();
        let builder = PromptBuilder::new(StyleConfig::new());
        let prompt = builder.phase_prompt(&episode, &ctx, &[]);

        assert!(
            prompt.contains("УТРОЕНИЕ: Этот эпизод содержит утроение"),
            "Expected triplication in Ru prompt"
        );

        let builder_en =
            PromptBuilder::new(StyleConfig::new()).with_lang(Lang::En);
        let prompt_en = builder_en.phase_prompt(&episode, &ctx, &[]);
        assert!(
            prompt_en.contains("TRIPLICATION: This episode uses triplication"),
            "Expected triplication in En prompt"
        );
    }

    #[test]
    fn test_character_prompt_multi_sphere_instruction() {
        let style = StyleConfig::new();
        let builder = PromptBuilder::new(style);

        // Tale with multi-sphere persona
        let mut tale = Tale::default();
        tale.personae.push(fableforge_core::Persona::new(
            1u32,
            vec![Sphere::Hero, Sphere::Dispatcher],
        ));
        tale.personae.push(fableforge_core::Persona::new(
            2u32,
            vec![Sphere::Villain],
        ));

        let prompt = builder.character_prompt(&tale);
        assert!(
            prompt.contains("совмещают несколько ролей"),
            "Expected duality instruction for multi-sphere personae in Ru prompt"
        );

        // English version
        let builder_en =
            PromptBuilder::new(StyleConfig::new()).with_lang(Lang::En);
        let prompt_en = builder_en.character_prompt(&tale);
        assert!(
            prompt_en.contains("combine multiple roles"),
            "Expected duality instruction for multi-sphere personae in En prompt"
        );
    }

    #[test]
    fn test_character_prompt_no_multi_sphere_instruction() {
        let builder = PromptBuilder::new(StyleConfig::new());

        // Tale with only single-sphere personae
        let mut tale = Tale::default();
        tale.personae.push(fableforge_core::Persona::new(
            1u32,
            vec![Sphere::Hero],
        ));
        tale.personae.push(fableforge_core::Persona::new(
            2u32,
            vec![Sphere::Villain],
        ));

        let prompt = builder.character_prompt(&tale);
        assert!(
            !prompt.contains("совмещают несколько ролей"),
            "Duality instruction should not appear when no multi-sphere personae"
        );
    }

    #[test]
    fn test_format_characters_includes_roles() {
        use fableforge_core::{Persona, PersonaId};

        use crate::context::{GeneratedCharacter, TaleContext};

        let mut ctx = TaleContext::new();
        ctx.characters.insert(
            PersonaId(1),
            GeneratedCharacter {
                id: PersonaId(1),
                name: "Кощей".to_string(),
                epithet: Some("тёмный чародей".to_string()),
                appearance: Some("высокий худой старик".to_string()),
            },
        );

        let personae = vec![Persona::new(
            1u32,
            vec![Sphere::Villain, Sphere::Donor],
        )];

        let builder = PromptBuilder::new(StyleConfig::new());
        let formatted = builder.format_characters(&ctx, &personae);

        assert!(
            formatted.contains("вредитель"),
            "Expected villain sphere in formatted characters"
        );
        assert!(
            formatted.contains("даритель"),
            "Expected donor sphere in formatted characters"
        );
    }
}
