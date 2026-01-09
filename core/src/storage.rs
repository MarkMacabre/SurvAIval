use crate::types::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::BTreeSet;
use std::path::Path;

pub struct Engine {
    conn: Connection,
}

impl Engine {
    pub fn open(db_path: &Path, read_only: bool) -> Result<Self, EngineError> {
        let conn = if read_only {
            Connection::open_with_flags(
                db_path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?
        } else {
            Connection::open(db_path)
                .map_err(|e| EngineError::DatabaseError(e.to_string()))?
        };

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;

        Ok(Engine { conn })
    }

    pub fn init_schema(&self) -> Result<(), EngineError> {
        let schema = include_str!("../../migrations/001_initial_schema.sql");
        self.conn
            .execute_batch(schema)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub fn insert_unit(&self, unit: &SurvivalUnitInput) -> Result<i64, EngineError> {
        // Parse enums
        let urgency = parse_urgency(&unit.urgency)?;
        let confidence = parse_confidence(&unit.confidence)?;
        let scenarios = parse_scenarios(&unit.scenario_tags)?;
        let environments = parse_environments(&unit.environment_tags)?;
        let tools = parse_tools(&unit.required_tools)?;

        // Validate
        if unit.immediate_action.is_empty() || unit.immediate_action.len() > 3 {
            return Err(EngineError::ValidationError(
                "Immediate action must have 1-3 steps".to_string(),
            ));
        }
        if scenarios.is_empty() {
            return Err(EngineError::ValidationError(
                "At least one scenario tag is required".to_string(),
            ));
        }
        if environments.is_empty() {
            return Err(EngineError::ValidationError(
                "At least one environment tag is required".to_string(),
            ));
        }

        // Serialize JSON fields
        let immediate_json = serde_json::to_string(&unit.immediate_action)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let expanded_json = serde_json::to_string(&unit.expanded_steps)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let warnings_json = serde_json::to_string(&unit.warnings)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let tools_json = serde_json::to_string(&tools)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let no_tool_json = unit
            .no_tool_fallback
            .as_ref()
            .map(|v| serde_json::to_string(v))
            .transpose()
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let failure_json = unit
            .failure_fallback
            .as_ref()
            .map(|v| serde_json::to_string(v))
            .transpose()
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;

        let created = chrono::Utc::now().to_rfc3339();

        // Insert main record
        self.conn
            .execute(
                "INSERT INTO survival_unit (
                    title, immediate_action_json, expanded_steps_json, warnings_json,
                    urgency, required_tools_json, no_tool_fallback_json, failure_fallback_json,
                    source_reference, confidence, created_iso8601
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    unit.title,
                    immediate_json,
                    expanded_json,
                    warnings_json,
                    urgency.to_i32(),
                    tools_json,
                    no_tool_json,
                    failure_json,
                    unit.source_reference,
                    confidence.to_i32(),
                    created,
                ],
            )
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;

        let unit_id = self.conn.last_insert_rowid();

        // Insert scenario tags
        for scenario in scenarios {
            self.conn
                .execute(
                    "INSERT INTO unit_scenario (unit_id, scenario) VALUES (?1, ?2)",
                    params![unit_id, scenario.to_i32()],
                )
                .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        }

        // Insert environment tags
        for environment in environments {
            self.conn
                .execute(
                    "INSERT INTO unit_environment (unit_id, environment) VALUES (?1, ?2)",
                    params![unit_id, environment.to_i32()],
                )
                .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        }

        Ok(unit_id)
    }

    pub fn get_unit_by_id(&self, id: i64) -> Result<SurvivalUnit, EngineError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, immediate_action_json, expanded_steps_json, warnings_json,
                        urgency, required_tools_json, no_tool_fallback_json, failure_fallback_json,
                        source_reference, confidence, created_iso8601
                 FROM survival_unit WHERE id = ?1",
            )
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;

        let unit = stmt
            .query_row(params![id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, String>(9)?,
                    row.get::<_, i32>(10)?,
                    row.get::<_, String>(11)?,
                ))
            })
            .optional()
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?
            .ok_or(EngineError::NotFound)?;

        let immediate_action: Vec<String> = serde_json::from_str(&unit.2)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let expanded_steps: Vec<String> = serde_json::from_str(&unit.3)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let warnings: Vec<String> = serde_json::from_str(&unit.4)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let required_tools: BTreeSet<Tool> = serde_json::from_str(&unit.6)
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let no_tool_fallback: Option<Vec<String>> = unit
            .7
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let failure_fallback: Option<Vec<String>> = unit
            .8
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;

        // Get scenario tags
        let mut scenario_stmt = self
            .conn
            .prepare("SELECT scenario FROM unit_scenario WHERE unit_id = ?1")
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let scenarios: BTreeSet<Scenario> = scenario_stmt
            .query_map(params![id], |row| row.get::<_, i32>(0))
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok().and_then(Scenario::from_i32))
            .collect();

        // Get environment tags
        let mut env_stmt = self
            .conn
            .prepare("SELECT environment FROM unit_environment WHERE unit_id = ?1")
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;
        let environments: BTreeSet<Environment> = env_stmt
            .query_map(params![id], |row| row.get::<_, i32>(0))
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?
            .filter_map(|r| r.ok().and_then(Environment::from_i32))
            .collect();

        Ok(SurvivalUnit {
            id: unit.0,
            title: unit.1,
            immediate_action,
            expanded_steps,
            warnings,
            scenario_tags: scenarios,
            environment_tags: environments,
            urgency: Urgency::from_i32(unit.5).ok_or(EngineError::DatabaseError(
                "Invalid urgency value".to_string(),
            ))?,
            required_tools,
            no_tool_fallback,
            failure_fallback,
            source_reference: unit.9,
            confidence: Confidence::from_i32(unit.10).ok_or(EngineError::DatabaseError(
                "Invalid confidence value".to_string(),
            ))?,
            created_iso8601: unit.11,
        })
    }

    pub fn query_candidates(
        &self,
        urgency: Urgency,
        scenarios: &BTreeSet<Scenario>,
        environments: &BTreeSet<Environment>,
    ) -> Result<Vec<SurvivalUnit>, EngineError> {
        let mut candidates = Vec::new();

        // Build query to filter by urgency
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT su.id
                 FROM survival_unit su
                 WHERE su.urgency = ?1",
            )
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?;

        let unit_ids: Vec<i64> = stmt
            .query_map(params![urgency.to_i32()], |row| row.get(0))
            .map_err(|e| EngineError::DatabaseError(e.to_string()))?
            .filter_map(Result::ok)
            .collect();

        for unit_id in unit_ids {
            let unit = self.get_unit_by_id(unit_id)?;

            // Check if any scenario matches
            let scenario_match = scenarios.iter().any(|s| unit.scenario_tags.contains(s));

            // Check if any environment matches
            let env_match = environments.iter().any(|e| unit.environment_tags.contains(e));

            if scenario_match && env_match {
                candidates.push(unit);
            }
        }

        Ok(candidates)
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

fn parse_urgency(s: &str) -> Result<Urgency, EngineError> {
    match s {
        "Immediate" => Ok(Urgency::Immediate),
        "High" => Ok(Urgency::High),
        "Medium" => Ok(Urgency::Medium),
        "Low" => Ok(Urgency::Low),
        _ => Err(EngineError::ValidationError(format!(
            "Invalid urgency: {}",
            s
        ))),
    }
}

fn parse_confidence(s: &str) -> Result<Confidence, EngineError> {
    match s {
        "Verified" => Ok(Confidence::Verified),
        "High" => Ok(Confidence::High),
        "Medium" => Ok(Confidence::Medium),
        "Low" => Ok(Confidence::Low),
        _ => Err(EngineError::ValidationError(format!(
            "Invalid confidence: {}",
            s
        ))),
    }
}

fn parse_scenarios(tags: &[String]) -> Result<BTreeSet<Scenario>, EngineError> {
    let mut set = BTreeSet::new();
    for tag in tags {
        let scenario = match tag.as_str() {
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
            _ => {
                return Err(EngineError::ValidationError(format!(
                    "Invalid scenario: {}",
                    tag
                )))
            }
        };
        set.insert(scenario);
    }
    Ok(set)
}

fn parse_environments(tags: &[String]) -> Result<BTreeSet<Environment>, EngineError> {
    let mut set = BTreeSet::new();
    for tag in tags {
        let environment = match tag.as_str() {
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
            _ => {
                return Err(EngineError::ValidationError(format!(
                    "Invalid environment: {}",
                    tag
                )))
            }
        };
        set.insert(environment);
    }
    Ok(set)
}

fn parse_tools(tags: &[String]) -> Result<BTreeSet<Tool>, EngineError> {
    let mut set = BTreeSet::new();
    for tag in tags {
        let tool = match tag.as_str() {
            "Gloves" => Tool::Gloves,
            "Tourniquet" => Tool::Tourniquet,
            "Bandage" => Tool::Bandage,
            "CPRMask" => Tool::CPRMask,
            "Splint" => Tool::Splint,
            "Rope" => Tool::Rope,
            "Knife" => Tool::Knife,
            "WaterFilter" => Tool::WaterFilter,
            "Firestarter" => Tool::Firestarter,
            "EmergencyBlanket" => Tool::EmergencyBlanket,
            "Whistle" => Tool::Whistle,
            "Flashlight" => Tool::Flashlight,
            "None" => Tool::None,
            _ => {
                return Err(EngineError::ValidationError(format!(
                    "Invalid tool: {}",
                    tag
                )))
            }
        };
        set.insert(tool);
    }
    Ok(set)
}
