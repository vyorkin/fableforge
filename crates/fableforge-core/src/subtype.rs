//! Function subtypes (подвиды функций) based on Propp's morphology.
//!
//! Each narrative function can have multiple subtypes representing specific
//! variations of the same plot element. For example, Villainy (A) has 19 subtypes:
//! A¹ — kidnapping, A² — theft of magical means, etc.

use crate::function::NarrativeFunction;
use crate::Lang;

/// Description of a function subtype.
#[derive(Debug, Clone, Copy)]
pub struct SubtypeInfo {
    /// Subtype index (1-based, as in Propp's notation).
    pub index: u8,
    /// Russian description.
    pub name_ru: &'static str,
    /// English description.
    pub name_en: &'static str,
}

impl SubtypeInfo {
    /// Get the name in the specified language.
    #[must_use]
    pub const fn name(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::Ru => self.name_ru,
            Lang::En => self.name_en,
        }
    }
}

/// Get all subtypes for a function.
#[must_use]
pub fn subtypes(function: NarrativeFunction) -> &'static [SubtypeInfo] {
    match function {
        NarrativeFunction::Absentation => ABSENTATION_SUBTYPES,
        NarrativeFunction::Interdiction => INTERDICTION_SUBTYPES,
        NarrativeFunction::Violation => VIOLATION_SUBTYPES,
        NarrativeFunction::Reconnaissance => RECONNAISSANCE_SUBTYPES,
        NarrativeFunction::Delivery => DELIVERY_SUBTYPES,
        NarrativeFunction::Trickery => TRICKERY_SUBTYPES,
        NarrativeFunction::Complicity => COMPLICITY_SUBTYPES,
        NarrativeFunction::Villainy => VILLAINY_SUBTYPES,
        NarrativeFunction::Lack => LACK_SUBTYPES,
        NarrativeFunction::Mediation => MEDIATION_SUBTYPES,
        NarrativeFunction::Counteraction => COUNTERACTION_SUBTYPES,
        NarrativeFunction::Departure => DEPARTURE_SUBTYPES,
        NarrativeFunction::DonorTest => DONOR_TEST_SUBTYPES,
        NarrativeFunction::HeroReaction => HERO_REACTION_SUBTYPES,
        NarrativeFunction::Acquisition => ACQUISITION_SUBTYPES,
        NarrativeFunction::Guidance => GUIDANCE_SUBTYPES,
        NarrativeFunction::Struggle => STRUGGLE_SUBTYPES,
        NarrativeFunction::Branding => BRANDING_SUBTYPES,
        NarrativeFunction::Victory => VICTORY_SUBTYPES,
        NarrativeFunction::Liquidation => LIQUIDATION_SUBTYPES,
        NarrativeFunction::Return => RETURN_SUBTYPES,
        NarrativeFunction::Pursuit => PURSUIT_SUBTYPES,
        NarrativeFunction::Rescue => RESCUE_SUBTYPES,
        NarrativeFunction::UnrecognizedArrival => UNRECOGNIZED_ARRIVAL_SUBTYPES,
        NarrativeFunction::UnfoundedClaims => UNFOUNDED_CLAIMS_SUBTYPES,
        NarrativeFunction::DifficultTask => DIFFICULT_TASK_SUBTYPES,
        NarrativeFunction::Solution => SOLUTION_SUBTYPES,
        NarrativeFunction::Recognition => RECOGNITION_SUBTYPES,
        NarrativeFunction::Exposure => EXPOSURE_SUBTYPES,
        NarrativeFunction::Transfiguration => TRANSFIGURATION_SUBTYPES,
        NarrativeFunction::Punishment => PUNISHMENT_SUBTYPES,
        NarrativeFunction::Wedding => WEDDING_SUBTYPES,
    }
}

/// Get a specific subtype by index (1-based).
#[must_use]
pub fn subtype(function: NarrativeFunction, index: u8) -> Option<&'static SubtypeInfo> {
    subtypes(function)
        .iter()
        .find(|s| s.index == index)
}

/// Get the number of subtypes for a function.
#[must_use]
pub fn subtype_count(function: NarrativeFunction) -> usize {
    subtypes(function).len()
}

// ============================================================================
// Subtype definitions based on Propp's "Morphology of the Folktale"
// ============================================================================

/// α — Absentation (Отлучка)
const ABSENTATION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Отлучка старших", name_en: "Departure of elders" },
    SubtypeInfo { index: 2, name_ru: "Смерть родителей", name_en: "Death of parents" },
    SubtypeInfo { index: 3, name_ru: "Отлучка младших", name_en: "Departure of younger members" },
];

/// β — Interdiction (Запрет)
const INTERDICTION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Запрет выходить из дома", name_en: "Prohibition to go outside" },
    SubtypeInfo { index: 2, name_ru: "Запрет открывать дверь/сундук", name_en: "Prohibition to open door/chest" },
    SubtypeInfo { index: 3, name_ru: "Запрет произносить слово/имя", name_en: "Prohibition to speak word/name" },
];

/// γ — Violation (Нарушение)
const VIOLATION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Нарушение запрета выходить", name_en: "Violation of going out prohibition" },
    SubtypeInfo { index: 2, name_ru: "Открытие запретного", name_en: "Opening the forbidden" },
    SubtypeInfo { index: 3, name_ru: "Произнесение запретного", name_en: "Speaking the forbidden" },
];

/// δ — Reconnaissance (Выведывание)
const RECONNAISSANCE_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Выведывание местопребывания", name_en: "Reconnaissance of location" },
    SubtypeInfo { index: 2, name_ru: "Выспрашивание о ценностях", name_en: "Inquiry about valuables" },
    SubtypeInfo { index: 3, name_ru: "Выведывание об уязвимости", name_en: "Reconnaissance of weakness" },
];

/// ε — Delivery (Выдача сведений)
const DELIVERY_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Прямой ответ на расспросы", name_en: "Direct answer to questions" },
    SubtypeInfo { index: 2, name_ru: "Невольная выдача", name_en: "Involuntary disclosure" },
    SubtypeInfo { index: 3, name_ru: "Предательская выдача", name_en: "Treacherous disclosure" },
];

/// η — Trickery (Подвох)
const TRICKERY_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Уговоры злодея", name_en: "Villain's persuasion" },
    SubtypeInfo { index: 2, name_ru: "Применение волшебных средств", name_en: "Use of magical means" },
    SubtypeInfo { index: 3, name_ru: "Обман или обманные действия", name_en: "Deception or deceitful actions" },
];

/// θ — Complicity (Пособничество)
const COMPLICITY_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Герой поддаётся уговорам", name_en: "Hero yields to persuasion" },
    SubtypeInfo { index: 2, name_ru: "Герой реагирует механически", name_en: "Hero reacts mechanically" },
    SubtypeInfo { index: 3, name_ru: "Герой обманут", name_en: "Hero is deceived" },
];

/// A — Villainy (Вредительство)
const VILLAINY_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Похищение человека", name_en: "Kidnapping" },
    SubtypeInfo { index: 2, name_ru: "Похищение волшебного средства", name_en: "Seizure of magical agent" },
    SubtypeInfo { index: 3, name_ru: "Расхищение или порча посевов", name_en: "Plundering or destruction of crops" },
    SubtypeInfo { index: 4, name_ru: "Похищение дневного света", name_en: "Theft of daylight" },
    SubtypeInfo { index: 5, name_ru: "Хищение в иных формах", name_en: "Theft in other forms" },
    SubtypeInfo { index: 6, name_ru: "Нанесение телесного повреждения", name_en: "Bodily injury" },
    SubtypeInfo { index: 7, name_ru: "Причинение внезапного исчезновения", name_en: "Causing sudden disappearance" },
    SubtypeInfo { index: 8, name_ru: "Требование или выманивание жертвы", name_en: "Demanding or luring victim" },
    SubtypeInfo { index: 9, name_ru: "Изгнание", name_en: "Expulsion" },
    SubtypeInfo { index: 10, name_ru: "Приказ бросить в воду", name_en: "Order to cast into water" },
    SubtypeInfo { index: 11, name_ru: "Околдование", name_en: "Casting a spell" },
    SubtypeInfo { index: 12, name_ru: "Подмена", name_en: "Substitution" },
    SubtypeInfo { index: 13, name_ru: "Приказ убить", name_en: "Order to kill" },
    SubtypeInfo { index: 14, name_ru: "Убийство", name_en: "Murder" },
    SubtypeInfo { index: 15, name_ru: "Заточение, тюремное заключение", name_en: "Imprisonment" },
    SubtypeInfo { index: 16, name_ru: "Угроза насильственного супружества", name_en: "Threat of forced marriage" },
    SubtypeInfo { index: 17, name_ru: "Угроза каннибализма", name_en: "Threat of cannibalism" },
    SubtypeInfo { index: 18, name_ru: "Мучение по ночам", name_en: "Tormenting at night" },
    SubtypeInfo { index: 19, name_ru: "Объявление войны", name_en: "Declaration of war" },
];

/// a — Lack (Недостача)
const LACK_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Недостача невесты или друга", name_en: "Lack of bride or friend" },
    SubtypeInfo { index: 2, name_ru: "Недостача волшебного средства", name_en: "Lack of magical agent" },
    SubtypeInfo { index: 3, name_ru: "Недостача диковинок", name_en: "Lack of wondrous objects" },
    SubtypeInfo { index: 4, name_ru: "Недостача яйца с Кощеевой смертью", name_en: "Lack of death-egg (life object)" },
    SubtypeInfo { index: 5, name_ru: "Нехватка денег, средств к существованию", name_en: "Lack of money or means" },
    SubtypeInfo { index: 6, name_ru: "Недостача в иных формах", name_en: "Lack in other forms" },
];

/// B — Mediation (Посредничество)
const MEDIATION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Клич, зов о помощи", name_en: "Call for help" },
    SubtypeInfo { index: 2, name_ru: "Отсылка героя напрямую", name_en: "Direct dispatch of hero" },
    SubtypeInfo { index: 3, name_ru: "Отпуск героя из дома", name_en: "Hero is permitted to depart" },
    SubtypeInfo { index: 4, name_ru: "Извещение о беде", name_en: "Announcement of misfortune" },
];

/// C — Counteraction (Начинающееся противодействие)
const COUNTERACTION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Герой-искатель соглашается", name_en: "Seeker-hero agrees" },
    SubtypeInfo { index: 2, name_ru: "Герой решает действовать сам", name_en: "Hero decides to act independently" },
];

/// ↑ — Departure (Отправка)
const DEPARTURE_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Герой отправляется из дома", name_en: "Hero leaves home" },
    SubtypeInfo { index: 2, name_ru: "Герой идёт за пострадавшим", name_en: "Hero follows the victim" },
];

/// D — Donor Test (Первая функция дарителя)
const DONOR_TEST_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Испытание едой или питьём", name_en: "Test by food or drink" },
    SubtypeInfo { index: 2, name_ru: "Приветствие, расспрос", name_en: "Greeting, questioning" },
    SubtypeInfo { index: 3, name_ru: "Просьба о помощи, услуге", name_en: "Request for help or service" },
    SubtypeInfo { index: 4, name_ru: "Просьба об освобождении пленника", name_en: "Request to free captive" },
    SubtypeInfo { index: 5, name_ru: "Просьба о разделе добычи", name_en: "Request to divide spoils" },
    SubtypeInfo { index: 6, name_ru: "Иные просьбы", name_en: "Other requests" },
    SubtypeInfo { index: 7, name_ru: "Враждебное нападение", name_en: "Hostile attack" },
    SubtypeInfo { index: 8, name_ru: "Загадка", name_en: "Riddle" },
    SubtypeInfo { index: 9, name_ru: "Предложение обмена", name_en: "Offer of exchange" },
];

/// E — Hero's Reaction (Реакция героя)
const HERO_REACTION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Выдерживает испытание", name_en: "Withstands test" },
    SubtypeInfo { index: 2, name_ru: "Отвечает на приветствие", name_en: "Responds to greeting" },
    SubtypeInfo { index: 3, name_ru: "Оказывает услугу", name_en: "Renders service" },
    SubtypeInfo { index: 4, name_ru: "Освобождает пленника", name_en: "Frees captive" },
    SubtypeInfo { index: 5, name_ru: "Щадит или примиряет спорящих", name_en: "Spares or reconciles disputants" },
    SubtypeInfo { index: 6, name_ru: "Обманывает дарителя", name_en: "Deceives donor" },
    SubtypeInfo { index: 7, name_ru: "Побеждает в схватке", name_en: "Wins in combat" },
    SubtypeInfo { index: 8, name_ru: "Разгадывает загадку", name_en: "Solves riddle" },
    SubtypeInfo { index: 9, name_ru: "Соглашается на обмен", name_en: "Agrees to exchange" },
];

/// F — Acquisition (Получение волшебного средства)
const ACQUISITION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Средство передаётся напрямую", name_en: "Agent is directly transferred" },
    SubtypeInfo { index: 2, name_ru: "Указывается местонахождение средства", name_en: "Agent's location is indicated" },
    SubtypeInfo { index: 3, name_ru: "Средство изготавливается", name_en: "Agent is prepared" },
    SubtypeInfo { index: 4, name_ru: "Средство продаётся, покупается", name_en: "Agent is sold/purchased" },
    SubtypeInfo { index: 5, name_ru: "Средство попадается случайно", name_en: "Agent is found by chance" },
    SubtypeInfo { index: 6, name_ru: "Средство появляется само", name_en: "Agent appears spontaneously" },
    SubtypeInfo { index: 7, name_ru: "Средство выпивается или съедается", name_en: "Agent is eaten or drunk" },
    SubtypeInfo { index: 8, name_ru: "Средство похищается", name_en: "Agent is seized" },
    SubtypeInfo { index: 9, name_ru: "Помощник сам предлагает услуги", name_en: "Helper offers services" },
];

/// G — Guidance (Пространственное перемещение)
const GUIDANCE_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Полёт по воздуху", name_en: "Flight through the air" },
    SubtypeInfo { index: 2, name_ru: "Передвижение по земле или воде", name_en: "Travel on land or water" },
    SubtypeInfo { index: 3, name_ru: "Герою указывают путь", name_en: "Hero is led" },
    SubtypeInfo { index: 4, name_ru: "Путь показывают следы", name_en: "Path is shown by tracks" },
    SubtypeInfo { index: 5, name_ru: "Использование неподвижных средств", name_en: "Use of stationary means" },
    SubtypeInfo { index: 6, name_ru: "Следование по кровавому следу", name_en: "Following a bloody trail" },
];

/// H — Struggle (Борьба)
const STRUGGLE_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Бой на открытом поле", name_en: "Fight in open field" },
    SubtypeInfo { index: 2, name_ru: "Соревнование", name_en: "Competition" },
    SubtypeInfo { index: 3, name_ru: "Игра в карты", name_en: "Card game" },
    SubtypeInfo { index: 4, name_ru: "Взвешивание", name_en: "Weighing" },
];

/// J — Branding (Клеймение)
const BRANDING_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Нанесение клейма на тело", name_en: "Mark applied to body" },
    SubtypeInfo { index: 2, name_ru: "Передача кольца или платка", name_en: "Ring or handkerchief given" },
];

/// I — Victory (Победа)
const VICTORY_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Победа в открытом бою", name_en: "Victory in open battle" },
    SubtypeInfo { index: 2, name_ru: "Победа в соревновании", name_en: "Victory in competition" },
    SubtypeInfo { index: 3, name_ru: "Выигрыш в карты", name_en: "Win in cards" },
    SubtypeInfo { index: 4, name_ru: "Победа при взвешивании", name_en: "Victory in weighing" },
    SubtypeInfo { index: 5, name_ru: "Победа без боя — убийство", name_en: "Victory without battle — killing" },
    SubtypeInfo { index: 6, name_ru: "Изгнание злодея", name_en: "Expulsion of villain" },
];

/// K — Liquidation of lack (Ликвидация недостачи)
const LIQUIDATION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Объект поисков добывается силой или хитростью", name_en: "Object obtained by force or cunning" },
    SubtypeInfo { index: 2, name_ru: "Объект указывается", name_en: "Object is pointed out" },
    SubtypeInfo { index: 3, name_ru: "Объект добывается несколькими помощниками", name_en: "Object obtained by several helpers" },
    SubtypeInfo { index: 4, name_ru: "Объект добывается волшебным средством", name_en: "Object obtained by magical agent" },
    SubtypeInfo { index: 5, name_ru: "Объект находится благодаря предыдущим действиям", name_en: "Object obtained as result of prior actions" },
    SubtypeInfo { index: 6, name_ru: "Бедность устраняется волшебным средством", name_en: "Poverty remedied by magical agent" },
    SubtypeInfo { index: 7, name_ru: "Объект захватывается", name_en: "Object is seized" },
    SubtypeInfo { index: 8, name_ru: "Снимается проклятие, заклятие", name_en: "Spell is broken" },
    SubtypeInfo { index: 9, name_ru: "Убитый оживает", name_en: "Slain person is revived" },
    SubtypeInfo { index: 10, name_ru: "Пленник освобождается", name_en: "Captive is freed" },
];

/// ↓ — Return (Возвращение)
const RETURN_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Возвращение по воздуху", name_en: "Return by air" },
    SubtypeInfo { index: 2, name_ru: "Возвращение по земле или воде", name_en: "Return by land or water" },
    SubtypeInfo { index: 3, name_ru: "Возвращение с провожатым", name_en: "Return with guide" },
    SubtypeInfo { index: 4, name_ru: "Бегство", name_en: "Flight (escape)" },
];

/// Pr — Pursuit (Преследование)
const PURSUIT_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Преследование по воздуху", name_en: "Pursuit through the air" },
    SubtypeInfo { index: 2, name_ru: "Преследование по земле", name_en: "Pursuit on land" },
    SubtypeInfo { index: 3, name_ru: "Преследование с превращениями", name_en: "Pursuit with transformations" },
    SubtypeInfo { index: 4, name_ru: "Преследование с препятствиями", name_en: "Pursuit with obstacles" },
    SubtypeInfo { index: 5, name_ru: "Преследование с попыткой проглотить", name_en: "Pursuit attempting to devour" },
    SubtypeInfo { index: 6, name_ru: "Преследование с попыткой погубить", name_en: "Pursuit attempting to destroy" },
];

/// Rs — Rescue (Спасение)
const RESCUE_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Бегство с превращениями", name_en: "Flight with transformations" },
    SubtypeInfo { index: 2, name_ru: "Бегство с бросанием предметов", name_en: "Flight throwing objects" },
    SubtypeInfo { index: 3, name_ru: "Бегство с укрывательством", name_en: "Flight with hiding" },
    SubtypeInfo { index: 4, name_ru: "Укрывание у кузнеца", name_en: "Hiding at blacksmith's" },
    SubtypeInfo { index: 5, name_ru: "Преследователь теряет силу", name_en: "Pursuer loses power" },
    SubtypeInfo { index: 6, name_ru: "Спасение от проглатывания", name_en: "Rescue from being devoured" },
];

/// O — Unrecognized Arrival (Неузнанное прибытие)
const UNRECOGNIZED_ARRIVAL_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Прибытие домой", name_en: "Arrival home" },
    SubtypeInfo { index: 2, name_ru: "Прибытие к иному царю", name_en: "Arrival at another king's court" },
];

/// L — Unfounded Claims (Необоснованные притязания)
const UNFOUNDED_CLAIMS_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Ложный герой — генерал, водовоз", name_en: "False hero — general, water-carrier" },
    SubtypeInfo { index: 2, name_ru: "Ложный герой — братья", name_en: "False hero — brothers" },
    SubtypeInfo { index: 3, name_ru: "Ложный герой — сёстры", name_en: "False hero — sisters" },
];

/// M — Difficult Task (Трудная задача)
const DIFFICULT_TASK_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Испытание едой или питьём", name_en: "Ordeal by food or drink" },
    SubtypeInfo { index: 2, name_ru: "Испытание огнём", name_en: "Ordeal by fire" },
    SubtypeInfo { index: 3, name_ru: "Загадки", name_en: "Riddles" },
    SubtypeInfo { index: 4, name_ru: "Испытание выбором", name_en: "Ordeal of choice" },
    SubtypeInfo { index: 5, name_ru: "Выполнение заказа", name_en: "Fulfillment of an order" },
    SubtypeInfo { index: 6, name_ru: "Иные задачи", name_en: "Other tasks" },
];

/// N — Solution (Решение)
const SOLUTION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Задача решается до срока", name_en: "Task solved before deadline" },
    SubtypeInfo { index: 2, name_ru: "Задача решается в срок", name_en: "Task solved on time" },
    SubtypeInfo { index: 3, name_ru: "Задача решается с помощью помощника", name_en: "Task solved with helper's aid" },
    SubtypeInfo { index: 4, name_ru: "Задача решается хитростью", name_en: "Task solved by cunning" },
];

/// Q — Recognition (Узнавание)
const RECOGNITION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Узнавание по клейму", name_en: "Recognition by mark" },
    SubtypeInfo { index: 2, name_ru: "Узнавание по предмету", name_en: "Recognition by token" },
    SubtypeInfo { index: 3, name_ru: "Узнавание по выполнению задачи", name_en: "Recognition by task accomplishment" },
];

/// Ex — Exposure (Обличение)
const EXPOSURE_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Обличение через рассказ", name_en: "Exposure through narrative" },
    SubtypeInfo { index: 2, name_ru: "Обличение через улики", name_en: "Exposure through evidence" },
];

/// T — Transfiguration (Трансфигурация)
const TRANSFIGURATION_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Новый облик в результате волшебства", name_en: "New appearance through magic" },
    SubtypeInfo { index: 2, name_ru: "Строительство дворца", name_en: "Building of palace" },
    SubtypeInfo { index: 3, name_ru: "Новые одежды", name_en: "New garments" },
    SubtypeInfo { index: 4, name_ru: "Комическая внешность", name_en: "Comic appearance" },
];

/// U — Punishment (Наказание)
const PUNISHMENT_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Расстрел", name_en: "Shooting" },
    SubtypeInfo { index: 2, name_ru: "Изгнание", name_en: "Banishment" },
    SubtypeInfo { index: 3, name_ru: "Привязывание к хвосту коня", name_en: "Tying to horse's tail" },
    SubtypeInfo { index: 4, name_ru: "Самоубийство", name_en: "Suicide" },
    SubtypeInfo { index: 5, name_ru: "Прощение", name_en: "Forgiveness" },
];

/// W — Wedding (Свадьба)
const WEDDING_SUBTYPES: &[SubtypeInfo] = &[
    SubtypeInfo { index: 1, name_ru: "Свадьба и воцарение", name_en: "Wedding and accession to throne" },
    SubtypeInfo { index: 2, name_ru: "Только свадьба", name_en: "Wedding only" },
    SubtypeInfo { index: 3, name_ru: "Только воцарение", name_en: "Accession to throne only" },
    SubtypeInfo { index: 4, name_ru: "Обещание брака", name_en: "Promise of marriage" },
    SubtypeInfo { index: 5, name_ru: "Возобновлённый брак", name_en: "Resumed marriage" },
    SubtypeInfo { index: 6, name_ru: "Денежное вознаграждение", name_en: "Monetary reward" },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_villainy_subtypes() {
        let subs = subtypes(NarrativeFunction::Villainy);
        assert_eq!(subs.len(), 19);
        assert_eq!(subs[0].index, 1);
        assert_eq!(subs[0].name_ru, "Похищение человека");
        assert_eq!(subs[0].name_en, "Kidnapping");
    }

    #[test]
    fn test_subtype_lookup() {
        let info = subtype(NarrativeFunction::DonorTest, 8).unwrap();
        assert_eq!(info.name_ru, "Загадка");
        assert_eq!(info.name_en, "Riddle");
    }

    #[test]
    fn test_subtype_not_found() {
        let info = subtype(NarrativeFunction::Villainy, 99);
        assert!(info.is_none());
    }

    #[test]
    fn test_subtype_count() {
        assert_eq!(subtype_count(NarrativeFunction::Villainy), 19);
        assert_eq!(subtype_count(NarrativeFunction::Lack), 6);
        assert_eq!(subtype_count(NarrativeFunction::DonorTest), 9);
    }

    #[test]
    fn test_all_functions_have_subtypes() {
        for func in NarrativeFunction::ALL {
            let count = subtype_count(func);
            assert!(count >= 1, "Function {:?} has no subtypes", func);
        }
    }
}
