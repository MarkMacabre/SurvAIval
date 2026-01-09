PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS survival_unit (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    immediate_action_json TEXT NOT NULL,
    expanded_steps_json TEXT NOT NULL,
    warnings_json TEXT NOT NULL,
    urgency INTEGER NOT NULL,
    required_tools_json TEXT NOT NULL,
    no_tool_fallback_json TEXT,
    failure_fallback_json TEXT,
    source_reference TEXT NOT NULL,
    confidence INTEGER NOT NULL,
    created_iso8601 TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS unit_scenario (
    unit_id INTEGER NOT NULL REFERENCES survival_unit(id) ON DELETE CASCADE,
    scenario INTEGER NOT NULL,
    PRIMARY KEY (unit_id, scenario)
);

CREATE TABLE IF NOT EXISTS unit_environment (
    unit_id INTEGER NOT NULL REFERENCES survival_unit(id) ON DELETE CASCADE,
    environment INTEGER NOT NULL,
    PRIMARY KEY (unit_id, environment)
);

CREATE VIRTUAL TABLE IF NOT EXISTS survival_unit_fts USING fts5(
    title, immediate_action, expanded_steps, warnings, content=''
);

CREATE TRIGGER unit_ai AFTER INSERT ON survival_unit BEGIN
    INSERT INTO survival_unit_fts(rowid, title, immediate_action, expanded_steps, warnings)
    VALUES (new.id, new.title, new.immediate_action_json, new.expanded_steps_json, new.warnings_json);
END;

CREATE TRIGGER unit_au AFTER UPDATE ON survival_unit BEGIN
    INSERT INTO survival_unit_fts(survival_unit_fts, rowid, title, immediate_action, expanded_steps, warnings)
    VALUES('delete', old.id, old.title, old.immediate_action_json, old.expanded_steps_json, old.warnings_json);
    INSERT INTO survival_unit_fts(rowid, title, immediate_action, expanded_steps, warnings)
    VALUES (new.id, new.title, new.immediate_action_json, new.expanded_steps_json, new.warnings_json);
END;

CREATE TRIGGER unit_ad AFTER DELETE ON survival_unit BEGIN
    INSERT INTO survival_unit_fts(survival_unit_fts, rowid, title, immediate_action, expanded_steps, warnings)
    VALUES('delete', old.id, old.title, old.immediate_action_json, old.expanded_steps_json, old.warnings_json);
END;
