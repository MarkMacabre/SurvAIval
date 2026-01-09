use crate::storage::Engine;
use crate::types::*;
use std::collections::{BTreeSet, HashMap};
use unicode_normalization::UnicodeNormalization;

pub struct Dictionaries {
    pub urgency_keywords: HashMap<Urgency, Vec<String>>,
    pub scenario_keywords: HashMap<Scenario, Vec<String>>,
    pub environment_keywords: HashMap<Environment, Vec<String>>,
    pub tool_keywords: HashMap<Tool, Vec<String>>,
    pub asr_corrections: HashMap<String, String>,
}

impl Dictionaries {
    pub fn load() -> Result<Self, EngineError> {
        let urgency_toml = include_str!("../../dictionaries/urgency_keywords.toml");
        let scenario_toml = include_str!("../../dictionaries/scenarios.toml");
        let environment_toml = include_str!("../../dictionaries/environments.toml");
        let tool_toml = include_str!("../../dictionaries/tools.toml");
        let asr_toml = include_str!("../../dictionaries/asr_corrections.toml");

        let urgency_keywords = parse_urgency_dict(urgency_toml)?;
        let scenario_keywords = parse_scenario_dict(scenario_toml)?;
        let environment_keywords = parse_environment_dict(environment_toml)?;
        let tool_keywords = parse_tool_dict(tool_toml)?;
        let asr_corrections = parse_asr_corrections(asr_toml)?;

        Ok(Dictionaries {
            urgency_keywords,
            scenario_keywords,
            environment_keywords,
            tool_keywords,
            asr_corrections,
        })
    }

    fn normalize_text(&self, text: &str) -> String {
        let mut normalized = text.nfc().collect::<String>().to_lowercase();

        // Apply ASR corrections
        for (wrong, correct) in &self.asr_corrections {
            normalized = normalized.replace(wrong, correct);
        }

        normalized
    }

    fn detect_urgency(&self, text: &str) -> Result<Urgency, EngineError> {
        let normalized = self.normalize_text(text);
        let mut matches = Vec::new();

        for (urgency, keywords) in &self.urgency_keywords {
            for keyword in keywords {
                if normalized.contains(keyword) {
                    matches.push(*urgency);
                    break;
                }
            }
        }

        if matches.is_empty() {
            return Err(EngineError::MissingUrgency);
        }

        // Return highest priority urgency (Immediate > High > Medium > Low)
        matches.sort();
        Ok(matches[0])
    }

    fn detect_scenarios(&self, text: &str) -> BTreeSet<Scenario> {
        let normalized = self.normalize_text(text);
        let mut matches = BTreeSet::new();

        // Sort keywords by length (longest first) for better phrase matching
        let mut all_keywords: Vec<(Scenario, &String)> = self
            .scenario_keywords
            .iter()
            .flat_map(|(scenario, keywords)| keywords.iter().map(move |k| (*scenario, k)))
            .collect();
        all_keywords.sort_by_key(|(_, k)| std::cmp::Reverse(k.len()));

        for (scenario, keyword) in all_keywords {
            if normalized.contains(keyword.as_str()) {
                matches.insert(scenario);
            }
        }

        matches
    }

    fn detect_environments(&self, text: &str) -> BTreeSet<Environment> {
        let normalized = self.normalize_text(text);
        let mut matches = BTreeSet::new();

        // Sort keywords by length (longest first)
        let mut all_keywords: Vec<(Environment, &String)> = self
            .environment_keywords
            .iter()
            .flat_map(|(env, keywords)| keywords.iter().map(move |k| (*env, k)))
            .collect();
        all_keywords.sort_by_key(|(_, k)| std::cmp::Reverse(k.len()));

        for (environment, keyword) in all_keywords {
            if normalized.contains(keyword.as_str()) {
                matches.insert(environment);
            }
        }

        matches
    }

    fn detect_tools(&self, text: &str) -> BTreeSet<Tool> {
        let normalized = self.normalize_text(text);
        let mut matches = BTreeSet::new();

        for (tool, keywords) in &self.tool_keywords {
            for keyword in keywords {
                if normalized.contains(keyword) {
                    matches.insert(*tool);
                    break;
                }
            }
        }

        matches
    }
}

pub fn query_best_match(
    engine: &Engine,
    context: &QueryContext,
    dicts: &Dictionaries,
) -> Result<BestMatch, EngineError> {
    let query_text = dicts.normalize_text(&context.query);

    // Detect urgency
    let urgency = if let Some(ref explicit) = context.explicit_urgency {
        match explicit.as_str() {
            "Immediate" => Urgency::Immediate,
            "High" => Urgency::High,
            "Medium" => Urgency::Medium,
            "Low" => Urgency::Low,
            _ => dicts.detect_urgency(&query_text)?,
        }
    } else {
        dicts.detect_urgency(&query_text)?
    };

    // Detect scenarios
    let scenarios = if let Some(ref explicit) = context.explicit_scenarios {
        parse_scenario_names(explicit)?
    } else {
        let detected = dicts.detect_scenarios(&query_text);
        if detected.is_empty() {
            return Err(EngineError::NoMatch);
        }
        detected
    };

    // Detect environments
    let environments = if let Some(ref explicit) = context.explicit_environments {
        parse_environment_names(explicit)?
    } else {
        let detected = dicts.detect_environments(&query_text);
        if detected.is_empty() {
            // Default to Urban and Wilderness if not specified
            let mut default = BTreeSet::new();
            default.insert(Environment::Urban);
            default.insert(Environment::Wilderness);
            default
        } else {
            detected
        }
    };

    // Detect available tools
    let mut available_tools = dicts.detect_tools(&query_text);
    for tool_name in &context.possible_tools {
        if let Some(tool) = parse_tool_name(tool_name) {
            available_tools.insert(tool);
        }
    }

    // Query candidates from database
    let candidates = engine.query_candidates(urgency, &scenarios, &environments)?;

    if candidates.is_empty() {
        return Err(EngineError::NoMatch);
    }

    // Score all candidates
    let mut scored: Vec<(i32, SurvivalUnit, Vec<String>)> = candidates
        .into_iter()
        .map(|unit| {
            let (score, rationale) = score_unit(&unit, &query_text, &available_tools, urgency);
            (score, unit, rationale)
        })
        .collect();

    // Sort by score (descending), then by tie-breakers
    scored.sort_by(|a, b| {
        let score_cmp = b.0.cmp(&a.0);
        if score_cmp != std::cmp::Ordering::Equal {
            return score_cmp;
        }

        // Tie-breaker 1: Higher confidence
        let conf_cmp = a.1.confidence.cmp(&b.1.confidence);
        if conf_cmp != std::cmp::Ordering::Equal {
            return conf_cmp;
        }

        // Tie-breaker 2: Fewer required tools
        let tools_cmp = a.1.required_tools.len().cmp(&b.1.required_tools.len());
        if tools_cmp != std::cmp::Ordering::Equal {
            return tools_cmp;
        }

        // Tie-breaker 3: Shorter immediate action
        let action_text_len_a: usize = a.1.immediate_action.iter().map(|s| s.len()).sum();
        let action_text_len_b: usize = b.1.immediate_action.iter().map(|s| s.len()).sum();
        let len_cmp = action_text_len_a.cmp(&action_text_len_b);
        if len_cmp != std::cmp::Ordering::Equal {
            return len_cmp;
        }

        // Tie-breaker 4: Lower unit ID
        a.1.id.cmp(&b.1.id)
    });

    // Apply safety policy
    let best = &scored[0];
    if best.1.confidence != Confidence::Verified {
        // Check if a Verified candidate is within 12 points
        for candidate in &scored[1..] {
            if candidate.1.confidence == Confidence::Verified {
                if best.0 - candidate.0 <= 12 {
                    // Prefer verified
                    return build_best_match(&candidate.1, candidate.0, &candidate.2, context);
                }
            }
        }
    }

    build_best_match(&best.1, best.0, &best.2, context)
}

fn score_unit(
    unit: &SurvivalUnit,
    query_text: &str,
    available_tools: &BTreeSet<Tool>,
    urgency: Urgency,
) -> (i32, Vec<String>) {
    let mut score = 0;
    let mut rationale = Vec::new();

    // Base score for urgency match
    score += 100;
    rationale.push(format!("Urgency:{:?}", urgency));

    // Tool score (0-30)
    let tool_score = calculate_tool_score(unit, available_tools);
    score += tool_score;
    rationale.push(format!("ToolMatch:+{}", tool_score));

    // Confidence score (0-40)
    let conf_score = unit.confidence.score();
    score += conf_score;
    rationale.push(format!("Confidence:+{}", conf_score));

    // Relevance score (0-25)
    let rel_score = calculate_relevance_score(unit, query_text);
    score += rel_score;
    rationale.push(format!("Relevance:+{}", rel_score));

    // Safety score (0-15)
    let safety_score = calculate_safety_score(unit, available_tools);
    score += safety_score;
    if safety_score > 0 {
        rationale.push(format!("Safety:+{}", safety_score));
    }

    (score, rationale)
}

fn calculate_tool_score(unit: &SurvivalUnit, available_tools: &BTreeSet<Tool>) -> i32 {
    if unit.required_tools.contains(&Tool::None) {
        return 30; // No tools required
    }

    let all_available = unit
        .required_tools
        .iter()
        .all(|t| available_tools.contains(t));

    if all_available {
        return 30; // All required tools available
    }

    if unit.no_tool_fallback.is_some() {
        let some_available = unit
            .required_tools
            .iter()
            .any(|t| available_tools.contains(t));
        if some_available {
            return 10; // Some tools + fallback
        }
        return 20; // No tools but fallback exists
    }

    0 // Missing tools, no fallback
}

fn calculate_relevance_score(unit: &SurvivalUnit, query_text: &str) -> i32 {
    let title_lower = unit.title.to_lowercase();
    let immediate_text = unit.immediate_action.join(" ").to_lowercase();

    // Check for exact phrase match
    let query_words: Vec<&str> = query_text.split_whitespace().collect();
    for window_size in (3..=query_words.len()).rev() {
        for window in query_words.windows(window_size) {
            let phrase = window.join(" ");
            if title_lower.contains(&phrase) || immediate_text.contains(&phrase) {
                return 20;
            }
        }
    }

    // Check for all tokens present
    let query_tokens: BTreeSet<&str> = query_words.iter().copied().collect();
    let title_tokens: BTreeSet<&str> = title_lower.split_whitespace().collect();
    let immediate_tokens: BTreeSet<&str> = immediate_text.split_whitespace().collect();

    let all_tokens_present = query_tokens
        .iter()
        .all(|t| title_tokens.contains(t) || immediate_tokens.contains(t));

    if all_tokens_present && query_tokens.len() > 2 {
        return 12;
    }

    // Check for some tokens present
    let some_tokens_present = query_tokens
        .iter()
        .any(|t| title_tokens.contains(t) || immediate_tokens.contains(t));

    if some_tokens_present {
        return 5;
    }

    0
}

fn calculate_safety_score(unit: &SurvivalUnit, available_tools: &BTreeSet<Tool>) -> i32 {
    let mut score = 0;

    if unit.failure_fallback.is_some() {
        score += 8;
    }

    let tools_missing = unit
        .required_tools
        .iter()
        .any(|t| !available_tools.contains(t) && *t != Tool::None);

    if tools_missing && unit.no_tool_fallback.is_some() {
        score += 5;
    }

    if !unit.warnings.is_empty() {
        score += 2;
    }

    score
}

fn build_best_match(
    unit: &SurvivalUnit,
    score: i32,
    rationale: &[String],
    context: &QueryContext,
) -> Result<BestMatch, EngineError> {
    let why_it_matters = if let Some(first_expanded) = unit.expanded_steps.first() {
        first_expanded.clone()
    } else {
        "Critical survival situation".to_string()
    };

    let next_step = if unit.expanded_steps.len() > 1 {
        unit.expanded_steps[1].clone()
    } else {
        "Monitor situation and seek help".to_string()
    };

    let fallback = unit
        .no_tool_fallback
        .as_ref()
        .or(unit.failure_fallback.as_ref())
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_else(|| "Seek professional help immediately".to_string());

    let confidence_str = format!("{:?}", unit.confidence);
    let mut message_suffix = String::new();

    if unit.confidence != Confidence::Verified && !context.require_verified {
        message_suffix = " I do not have verified information for this scenario.".to_string();
    }

    Ok(BestMatch {
        status: "ok".to_string(),
        unit: BestMatchUnit {
            id: unit.id,
            title: unit.title.clone() + &message_suffix,
            immediate_action: unit.immediate_action.clone(),
            warnings: unit.warnings.clone(),
            why_it_matters,
            next_step,
            fallback,
            confidence: confidence_str,
            source_reference: unit.source_reference.clone(),
        },
        score,
        rationale: rationale.to_vec(),
    })
}

fn parse_urgency_dict(toml: &str) -> Result<HashMap<Urgency, Vec<String>>, EngineError> {
    let value: toml::Value = toml::from_str(toml)
        .map_err(|e| EngineError::DatabaseError(format!("Failed to parse urgency TOML: {}", e)))?;

    let mut map = HashMap::new();

    if let Some(immediate) = value.get("Immediate").and_then(|v| v.get("keywords")) {
        map.insert(
            Urgency::Immediate,
            immediate
                .as_array()
                .ok_or(EngineError::DatabaseError("Invalid TOML format".to_string()))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
        );
    }

    if let Some(high) = value.get("High").and_then(|v| v.get("keywords")) {
        map.insert(
            Urgency::High,
            high.as_array()
                .ok_or(EngineError::DatabaseError("Invalid TOML format".to_string()))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
        );
    }

    if let Some(medium) = value.get("Medium").and_then(|v| v.get("keywords")) {
        map.insert(
            Urgency::Medium,
            medium
                .as_array()
                .ok_or(EngineError::DatabaseError("Invalid TOML format".to_string()))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
        );
    }

    if let Some(low) = value.get("Low").and_then(|v| v.get("keywords")) {
        map.insert(
            Urgency::Low,
            low.as_array()
                .ok_or(EngineError::DatabaseError("Invalid TOML format".to_string()))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
        );
    }

    Ok(map)
}

fn parse_scenario_dict(toml: &str) -> Result<HashMap<Scenario, Vec<String>>, EngineError> {
    let value: toml::Value = toml::from_str(toml)
        .map_err(|e| EngineError::DatabaseError(format!("Failed to parse scenario TOML: {}", e)))?;

    let scenarios = [
        ("Bleeding", Scenario::Bleeding),
        ("CardiacArrest", Scenario::CardiacArrest),
        ("Choking", Scenario::Choking),
        ("Drowning", Scenario::Drowning),
        ("Fracture", Scenario::Fracture),
        ("Burn", Scenario::Burn),
        ("HeatStroke", Scenario::HeatStroke),
        ("Hypothermia", Scenario::Hypothermia),
        ("Poisoning", Scenario::Poisoning),
        ("AnaphylaxisAllergy", Scenario::AnaphylaxisAllergy),
        ("SnakeBite", Scenario::SnakeBite),
        ("Shock", Scenario::Shock),
        ("Seizure", Scenario::Seizure),
        ("WoundInfection", Scenario::WoundInfection),
        ("Dehydration", Scenario::Dehydration),
    ];

    let mut map = HashMap::new();
    for (name, scenario) in scenarios {
        if let Some(keywords) = value.get(name).and_then(|v| v.get("keywords")) {
            map.insert(
                scenario,
                keywords
                    .as_array()
                    .ok_or(EngineError::DatabaseError("Invalid TOML format".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
            );
        }
    }

    Ok(map)
}

fn parse_environment_dict(toml: &str) -> Result<HashMap<Environment, Vec<String>>, EngineError> {
    let value: toml::Value = toml::from_str(toml).map_err(|e| {
        EngineError::DatabaseError(format!("Failed to parse environment TOML: {}", e))
    })?;

    let environments = [
        ("Urban", Environment::Urban),
        ("Wilderness", Environment::Wilderness),
        ("Mountain", Environment::Mountain),
        ("Desert", Environment::Desert),
        ("Coastal", Environment::Coastal),
        ("ColdWeather", Environment::ColdWeather),
        ("HotWeather", Environment::HotWeather),
        ("Subterranean", Environment::Subterranean),
        ("ConfinedSpaces", Environment::ConfinedSpaces),
        ("Aquatic", Environment::Aquatic),
    ];

    let mut map = HashMap::new();
    for (name, environment) in environments {
        if let Some(keywords) = value.get(name).and_then(|v| v.get("keywords")) {
            map.insert(
                environment,
                keywords
                    .as_array()
                    .ok_or(EngineError::DatabaseError("Invalid TOML format".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
            );
        }
    }

    Ok(map)
}

fn parse_tool_dict(toml: &str) -> Result<HashMap<Tool, Vec<String>>, EngineError> {
    let value: toml::Value = toml::from_str(toml)
        .map_err(|e| EngineError::DatabaseError(format!("Failed to parse tool TOML: {}", e)))?;

    let tools = [
        ("Gloves", Tool::Gloves),
        ("Tourniquet", Tool::Tourniquet),
        ("Bandage", Tool::Bandage),
        ("CPRMask", Tool::CPRMask),
        ("Splint", Tool::Splint),
        ("Rope", Tool::Rope),
        ("Knife", Tool::Knife),
        ("WaterFilter", Tool::WaterFilter),
        ("Firestarter", Tool::Firestarter),
        ("EmergencyBlanket", Tool::EmergencyBlanket),
        ("Whistle", Tool::Whistle),
        ("Flashlight", Tool::Flashlight),
        ("None", Tool::None),
    ];

    let mut map = HashMap::new();
    for (name, tool) in tools {
        if let Some(keywords) = value.get(name).and_then(|v| v.get("keywords")) {
            map.insert(
                tool,
                keywords
                    .as_array()
                    .ok_or(EngineError::DatabaseError("Invalid TOML format".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
            );
        }
    }

    Ok(map)
}

fn parse_asr_corrections(toml: &str) -> Result<HashMap<String, String>, EngineError> {
    let value: toml::Value = toml::from_str(toml)
        .map_err(|e| EngineError::DatabaseError(format!("Failed to parse ASR TOML: {}", e)))?;

    let corrections = value
        .get("corrections")
        .and_then(|v| v.as_table())
        .ok_or(EngineError::DatabaseError(
            "Missing corrections table".to_string(),
        ))?;

    let mut map = HashMap::new();
    for (key, value) in corrections {
        if let Some(corrected) = value.as_str() {
            map.insert(key.clone(), corrected.to_string());
        }
    }

    Ok(map)
}

fn parse_scenario_names(names: &[String]) -> Result<BTreeSet<Scenario>, EngineError> {
    let mut set = BTreeSet::new();
    for name in names {
        let scenario = match name.as_str() {
            "Bleeding" => Scenario::Bleeding,
            "CardiacArrest" => Scenario::CardiacArrest,
            "Choking" => Scenario::Choking,
            "Drowning" => Scenario::Drowning,
            "Fracture" => Scenario::Fracture,
            "Burn" => Scenario::Burn,
            "HeatStroke" => Scenario::HeatStroke,
            "Hypothermia" => Scenario::Hypothermia,
            "Poisoning" => Scenario::Poisoning,
            "AnaphylaxisAllergy" => Scenario::AnaphylaxisAllergy,
            "SnakeBite" => Scenario::SnakeBite,
            "Shock" => Scenario::Shock,
            "Seizure" => Scenario::Seizure,
            "WoundInfection" => Scenario::WoundInfection,
            "Dehydration" => Scenario::Dehydration,
            _ => return Err(EngineError::ValidationError(format!("Invalid scenario: {}", name))),
        };
        set.insert(scenario);
    }
    Ok(set)
}

fn parse_environment_names(names: &[String]) -> Result<BTreeSet<Environment>, EngineError> {
    let mut set = BTreeSet::new();
    for name in names {
        let environment = match name.as_str() {
            "Urban" => Environment::Urban,
            "Wilderness" => Environment::Wilderness,
            "Mountain" => Environment::Mountain,
            "Desert" => Environment::Desert,
            "Coastal" => Environment::Coastal,
            "ColdWeather" => Environment::ColdWeather,
            "HotWeather" => Environment::HotWeather,
            "Subterranean" => Environment::Subterranean,
            "ConfinedSpaces" => Environment::ConfinedSpaces,
            "Aquatic" => Environment::Aquatic,
            _ => return Err(EngineError::ValidationError(format!("Invalid environment: {}", name))),
        };
        set.insert(environment);
    }
    Ok(set)
}

fn parse_tool_name(name: &str) -> Option<Tool> {
    match name {
        "Gloves" => Some(Tool::Gloves),
        "Tourniquet" => Some(Tool::Tourniquet),
        "Bandage" => Some(Tool::Bandage),
        "CPRMask" => Some(Tool::CPRMask),
        "Splint" => Some(Tool::Splint),
        "Rope" => Some(Tool::Rope),
        "Knife" => Some(Tool::Knife),
        "WaterFilter" => Some(Tool::WaterFilter),
        "Firestarter" => Some(Tool::Firestarter),
        "EmergencyBlanket" => Some(Tool::EmergencyBlanket),
        "Whistle" => Some(Tool::Whistle),
        "Flashlight" => Some(Tool::Flashlight),
        "None" => Some(Tool::None),
        _ => None,
    }
}
