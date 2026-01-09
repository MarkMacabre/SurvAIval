pub mod ffi;
pub mod ingest;
pub mod search;
pub mod storage;
pub mod types;

use std::path::Path;
pub use types::*;

use search::Dictionaries;
use storage::Engine;

// === Lifecycle ===

pub fn open_engine(db_path: &Path, read_only: bool) -> Result<Engine, EngineError> {
    Engine::open(db_path, read_only)
}

pub fn close_engine(_engine: Engine) -> Result<(), EngineError> {
    // Engine will be dropped automatically
    Ok(())
}

// === Ingestion ===

pub fn ingest_survival_unit(
    engine: &Engine,
    unit: SurvivalUnitInput,
) -> Result<SurvivalUnit, EngineError> {
    ingest::ingest_survival_unit(engine, unit)
}

pub fn ingest_from_json_file(
    engine: &Engine,
    path: &Path,
) -> Result<IngestReport, EngineError> {
    ingest::ingest_from_json_file(engine, path)
}

pub fn validate_unit_input(input: &SurvivalUnitInput) -> Result<(), ValidationError> {
    ingest::validate_unit_input(input)
}

// === Retrieval ===

pub fn query_best_match(
    engine: &Engine,
    context: &QueryContext,
) -> Result<BestMatch, EngineError> {
    let dicts = Dictionaries::load()?;
    search::query_best_match(engine, context, &dicts)
}

pub fn get_unit_by_id(engine: &Engine, id: i64) -> Result<SurvivalUnit, EngineError> {
    engine.get_unit_by_id(id)
}

// === Database initialization ===

pub fn init_database(db_path: &Path) -> Result<(), EngineError> {
    let engine = Engine::open(db_path, false)?;
    engine.init_schema()?;
    Ok(())
}
