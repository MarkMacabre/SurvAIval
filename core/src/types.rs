use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Urgency {
    Immediate,
    High,
    Medium,
    Low,
}

impl Urgency {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Urgency::Immediate),
            1 => Some(Urgency::High),
            2 => Some(Urgency::Medium),
            3 => Some(Urgency::Low),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Urgency::Immediate => 0,
            Urgency::High => 1,
            Urgency::Medium => 2,
            Urgency::Low => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Confidence {
    Verified,
    High,
    Medium,
    Low,
}

impl Confidence {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Confidence::Verified),
            1 => Some(Confidence::High),
            2 => Some(Confidence::Medium),
            3 => Some(Confidence::Low),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Confidence::Verified => 0,
            Confidence::High => 1,
            Confidence::Medium => 2,
            Confidence::Low => 3,
        }
    }

    pub fn score(&self) -> i32 {
        match self {
            Confidence::Verified => 40,
            Confidence::High => 30,
            Confidence::Medium => 15,
            Confidence::Low => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Scenario {
    Bleeding,
    CardiacArrest,
    Choking,
    Drowning,
    Fracture,
    Burn,
    HeatStroke,
    Hypothermia,
    Poisoning,
    AnaphylaxisAllergy,
    SnakeBite,
    Shock,
    Seizure,
    WoundInfection,
    Dehydration,
}

impl Scenario {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Scenario::Bleeding),
            1 => Some(Scenario::CardiacArrest),
            2 => Some(Scenario::Choking),
            3 => Some(Scenario::Drowning),
            4 => Some(Scenario::Fracture),
            5 => Some(Scenario::Burn),
            6 => Some(Scenario::HeatStroke),
            7 => Some(Scenario::Hypothermia),
            8 => Some(Scenario::Poisoning),
            9 => Some(Scenario::AnaphylaxisAllergy),
            10 => Some(Scenario::SnakeBite),
            11 => Some(Scenario::Shock),
            12 => Some(Scenario::Seizure),
            13 => Some(Scenario::WoundInfection),
            14 => Some(Scenario::Dehydration),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Scenario::Bleeding => 0,
            Scenario::CardiacArrest => 1,
            Scenario::Choking => 2,
            Scenario::Drowning => 3,
            Scenario::Fracture => 4,
            Scenario::Burn => 5,
            Scenario::HeatStroke => 6,
            Scenario::Hypothermia => 7,
            Scenario::Poisoning => 8,
            Scenario::AnaphylaxisAllergy => 9,
            Scenario::SnakeBite => 10,
            Scenario::Shock => 11,
            Scenario::Seizure => 12,
            Scenario::WoundInfection => 13,
            Scenario::Dehydration => 14,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Environment {
    Urban,
    Wilderness,
    Mountain,
    Desert,
    Coastal,
    ColdWeather,
    HotWeather,
    Subterranean,
    ConfinedSpaces,
    Aquatic,
}

impl Environment {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Environment::Urban),
            1 => Some(Environment::Wilderness),
            2 => Some(Environment::Mountain),
            3 => Some(Environment::Desert),
            4 => Some(Environment::Coastal),
            5 => Some(Environment::ColdWeather),
            6 => Some(Environment::HotWeather),
            7 => Some(Environment::Subterranean),
            8 => Some(Environment::ConfinedSpaces),
            9 => Some(Environment::Aquatic),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Environment::Urban => 0,
            Environment::Wilderness => 1,
            Environment::Mountain => 2,
            Environment::Desert => 3,
            Environment::Coastal => 4,
            Environment::ColdWeather => 5,
            Environment::HotWeather => 6,
            Environment::Subterranean => 7,
            Environment::ConfinedSpaces => 8,
            Environment::Aquatic => 9,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Tool {
    Gloves,
    Tourniquet,
    Bandage,
    CPRMask,
    Splint,
    Rope,
    Knife,
    WaterFilter,
    Firestarter,
    EmergencyBlanket,
    Whistle,
    Flashlight,
    None,
}

impl Tool {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Tool::Gloves),
            1 => Some(Tool::Tourniquet),
            2 => Some(Tool::Bandage),
            3 => Some(Tool::CPRMask),
            4 => Some(Tool::Splint),
            5 => Some(Tool::Rope),
            6 => Some(Tool::Knife),
            7 => Some(Tool::WaterFilter),
            8 => Some(Tool::Firestarter),
            9 => Some(Tool::EmergencyBlanket),
            10 => Some(Tool::Whistle),
            11 => Some(Tool::Flashlight),
            12 => Some(Tool::None),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Tool::Gloves => 0,
            Tool::Tourniquet => 1,
            Tool::Bandage => 2,
            Tool::CPRMask => 3,
            Tool::Splint => 4,
            Tool::Rope => 5,
            Tool::Knife => 6,
            Tool::WaterFilter => 7,
            Tool::Firestarter => 8,
            Tool::EmergencyBlanket => 9,
            Tool::Whistle => 10,
            Tool::Flashlight => 11,
            Tool::None => 12,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurvivalUnit {
    pub id: i64,
    pub title: String,
    pub immediate_action: Vec<String>,
    pub expanded_steps: Vec<String>,
    pub warnings: Vec<String>,
    pub scenario_tags: BTreeSet<Scenario>,
    pub environment_tags: BTreeSet<Environment>,
    pub urgency: Urgency,
    pub required_tools: BTreeSet<Tool>,
    pub no_tool_fallback: Option<Vec<String>>,
    pub failure_fallback: Option<Vec<String>>,
    pub source_reference: String,
    pub confidence: Confidence,
    pub created_iso8601: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurvivalUnitInput {
    pub title: String,
    pub immediate_action: Vec<String>,
    pub expanded_steps: Vec<String>,
    pub warnings: Vec<String>,
    pub scenario_tags: Vec<String>,
    pub environment_tags: Vec<String>,
    pub urgency: String,
    pub required_tools: Vec<String>,
    pub no_tool_fallback: Option<Vec<String>>,
    pub failure_fallback: Option<Vec<String>>,
    pub source_reference: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContext {
    pub query: String,
    #[serde(default)]
    pub possible_tools: Vec<String>,
    pub explicit_urgency: Option<String>,
    pub explicit_scenarios: Option<Vec<String>>,
    pub explicit_environments: Option<Vec<String>>,
    #[serde(default)]
    pub require_verified: bool,
    pub compass_azimuth: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestMatch {
    pub status: String,
    pub unit: BestMatchUnit,
    pub score: i32,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestMatchUnit {
    pub id: i64,
    pub title: String,
    pub immediate_action: Vec<String>,
    pub warnings: Vec<String>,
    pub why_it_matters: String,
    pub next_step: String,
    pub fallback: String,
    pub confidence: String,
    pub source_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error_code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestReport {
    pub ingested: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub enum EngineError {
    DatabaseError(String),
    ValidationError(String),
    NotFound,
    NoMatch,
    MissingUrgency,
    AmbiguousScenario(Vec<Scenario>),
    AmbiguousEnvironment(Vec<Environment>),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::DatabaseError(s) => write!(f, "Database error: {}", s),
            EngineError::ValidationError(s) => write!(f, "Validation error: {}", s),
            EngineError::NotFound => write!(f, "Not found"),
            EngineError::NoMatch => write!(f, "I do not have verified information for this scenario."),
            EngineError::MissingUrgency => write!(f, "Urgency unclear — specify: Immediate / High / Medium / Low"),
            EngineError::AmbiguousScenario(scenarios) => {
                write!(f, "Scenario unclear — select one: {:?}", scenarios)
            }
            EngineError::AmbiguousEnvironment(envs) => {
                write!(f, "Environment unclear — select one: {:?}", envs)
            }
        }
    }
}

impl std::error::Error for EngineError {}

impl From<ValidationError> for EngineError {
    fn from(err: ValidationError) -> Self {
        EngineError::ValidationError(err.to_string())
    }
}

#[derive(Debug)]
pub enum ValidationError {
    MissingField(String),
    InvalidEnum(String),
    TooManySteps,
    NoScenario,
    NoEnvironment,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingField(field) => write!(f, "Missing required field: {}", field),
            ValidationError::InvalidEnum(field) => write!(f, "Invalid enum value: {}", field),
            ValidationError::TooManySteps => write!(f, "Immediate action must have 1-3 steps only"),
            ValidationError::NoScenario => write!(f, "At least one scenario tag is required"),
            ValidationError::NoEnvironment => write!(f, "At least one environment tag is required"),
        }
    }
}

impl std::error::Error for ValidationError {}
