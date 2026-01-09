use crate::storage::Engine;
use crate::types::*;
use std::fs;
use std::path::Path;

pub fn ingest_survival_unit(
    engine: &Engine,
    unit: SurvivalUnitInput,
) -> Result<SurvivalUnit, EngineError> {
    // Validate first
    validate_unit_input(&unit)?;

    // Insert into database
    let unit_id = engine.insert_unit(&unit)?;

    // Retrieve and return the complete unit
    engine.get_unit_by_id(unit_id)
}

pub fn ingest_from_json_file(
    engine: &Engine,
    path: &Path,
) -> Result<IngestReport, EngineError> {
    let contents = fs::read_to_string(path)
        .map_err(|e| EngineError::DatabaseError(format!("Failed to read file: {}", e)))?;

    let units: Vec<SurvivalUnitInput> = serde_json::from_str(&contents)
        .map_err(|e| EngineError::DatabaseError(format!("Failed to parse JSON: {}", e)))?;

    let mut ingested = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for (idx, unit) in units.into_iter().enumerate() {
        match ingest_survival_unit(engine, unit) {
            Ok(_) => ingested += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("Unit {}: {}", idx + 1, e));
            }
        }
    }

    Ok(IngestReport {
        ingested,
        failed,
        errors,
    })
}

pub fn validate_unit_input(input: &SurvivalUnitInput) -> Result<(), ValidationError> {
    // Check required fields
    if input.title.trim().is_empty() {
        return Err(ValidationError::MissingField("title".to_string()));
    }

    if input.source_reference.trim().is_empty() {
        return Err(ValidationError::MissingField(
            "source_reference".to_string(),
        ));
    }

    // Check immediate action count
    if input.immediate_action.is_empty() {
        return Err(ValidationError::MissingField(
            "immediate_action".to_string(),
        ));
    }

    if input.immediate_action.len() > 3 {
        return Err(ValidationError::TooManySteps);
    }

    // Check scenario tags
    if input.scenario_tags.is_empty() {
        return Err(ValidationError::NoScenario);
    }

    // Validate scenario enum values
    for tag in &input.scenario_tags {
        match tag.as_str() {
            "Bleeding" | "CardiacArrest" | "Choking" | "Drowning" | "Fracture" | "Burn"
            | "HeatStroke" | "Hypothermia" | "Poisoning" | "AnaphylaxisAllergy" | "SnakeBite"
            | "Shock" | "Seizure" | "WoundInfection" | "Dehydration" => {}
            _ => return Err(ValidationError::InvalidEnum(format!("scenario: {}", tag))),
        }
    }

    // Check environment tags
    if input.environment_tags.is_empty() {
        return Err(ValidationError::NoEnvironment);
    }

    // Validate environment enum values
    for tag in &input.environment_tags {
        match tag.as_str() {
            "Urban" | "Wilderness" | "Mountain" | "Desert" | "Coastal" | "ColdWeather"
            | "HotWeather" | "Subterranean" | "ConfinedSpaces" | "Aquatic" => {}
            _ => return Err(ValidationError::InvalidEnum(format!("environment: {}", tag))),
        }
    }

    // Validate urgency enum
    match input.urgency.as_str() {
        "Immediate" | "High" | "Medium" | "Low" => {}
        _ => {
            return Err(ValidationError::InvalidEnum(format!(
                "urgency: {}",
                input.urgency
            )))
        }
    }

    // Validate confidence enum
    match input.confidence.as_str() {
        "Verified" | "High" | "Medium" | "Low" => {}
        _ => {
            return Err(ValidationError::InvalidEnum(format!(
                "confidence: {}",
                input.confidence
            )))
        }
    }

    // Validate tool enum values
    for tag in &input.required_tools {
        match tag.as_str() {
            "Gloves" | "Tourniquet" | "Bandage" | "CPRMask" | "Splint" | "Rope" | "Knife"
            | "WaterFilter" | "Firestarter" | "EmergencyBlanket" | "Whistle" | "Flashlight"
            | "None" => {}
            _ => return Err(ValidationError::InvalidEnum(format!("tool: {}", tag))),
        }
    }

    Ok(())
}
