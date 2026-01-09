use clap::{Parser, Subcommand};
use std::path::PathBuf;
use survaival_core::*;

#[derive(Parser)]
#[command(name = "survaival")]
#[command(about = "Local-first offline survival knowledge system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new survival knowledge database
    Init {
        /// Path to the database file
        #[arg(long)]
        db: PathBuf,
    },
    /// Ingest survival units from a JSON file
    Ingest {
        /// Path to the database file
        #[arg(long)]
        db: PathBuf,
        /// Path to the JSON file containing survival units
        #[arg(long)]
        file: PathBuf,
    },
    /// Query for the best matching survival guidance
    Query {
        /// Path to the database file
        #[arg(long)]
        db: PathBuf,
        /// The query text describing the emergency
        query: String,
        /// Available tools (comma-separated)
        #[arg(long)]
        tools: Option<String>,
        /// Explicit urgency level (Immediate, High, Medium, Low)
        #[arg(long)]
        urgency: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { db } => {
            match init_database(&db) {
                Ok(_) => {
                    println!("✓ Database initialized at: {}", db.display());
                    println!("\nNext steps:");
                    println!("  1. Review and ingest verified survival units:");
                    println!(
                        "     survaival ingest --db {} --file <units.json>",
                        db.display()
                    );
                    println!("  2. Query for guidance:");
                    println!(
                        "     survaival query --db {} \"bleeding now woods\"",
                        db.display()
                    );
                }
                Err(e) => {
                    eprintln!("✗ Failed to initialize database: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Ingest { db, file } => {
            let engine = match open_engine(&db, false) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("✗ Failed to open database: {}", err);
                    std::process::exit(1);
                }
            };

            match ingest_from_json_file(&engine, &file) {
                Ok(report) => {
                    println!("✓ Ingestion complete");
                    println!("  Ingested: {}", report.ingested);
                    println!("  Failed: {}", report.failed);
                    if !report.errors.is_empty() {
                        println!("\nErrors:");
                        for error in report.errors {
                            println!("  - {}", error);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Failed to ingest: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Query {
            db,
            query,
            tools,
            urgency,
        } => {
            let engine = match open_engine(&db, true) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("✗ Failed to open database: {}", err);
                    std::process::exit(1);
                }
            };

            let possible_tools = tools
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let context = QueryContext {
                query: query.clone(),
                possible_tools,
                explicit_urgency: urgency,
                explicit_scenarios: None,
                explicit_environments: None,
                require_verified: false,
                compass_azimuth: None,
            };

            match query_best_match(&engine, &context) {
                Ok(result) => {
                    print_result(&result);
                }
                Err(e) => {
                    eprintln!("✗ Query failed: {}", e);
                    eprintln!("\nSeek professional help now and upload verified guidance to SurvAIval for future use.");
                    std::process::exit(1);
                }
            }
        }
    }
}

fn print_result(result: &BestMatch) {
    println!("\n{}", "═".repeat(80));
    println!("  {}", result.unit.title);
    println!("  Confidence: {} | Score: {}", result.unit.confidence, result.score);
    println!("{}", "═".repeat(80));

    println!("\n▶ IMMEDIATE ACTION:");
    for (i, step) in result.unit.immediate_action.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }

    if !result.unit.warnings.is_empty() {
        println!("\n⚠ WHAT NOT TO DO:");
        for warning in &result.unit.warnings {
            println!("  • {}", warning);
        }
    }

    println!("\n▶ WHY IT MATTERS:");
    println!("  {}", result.unit.why_it_matters);

    println!("\n▶ NEXT STEP:");
    println!("  {}", result.unit.next_step);

    println!("\n▶ FALLBACK:");
    println!("  {}", result.unit.fallback);

    println!("\n▶ SOURCE:");
    println!("  {}", result.unit.source_reference);

    if !result.rationale.is_empty() {
        println!("\n▶ MATCH RATIONALE:");
        for reason in &result.rationale {
            println!("  • {}", reason);
        }
    }

    println!("\n{}", "═".repeat(80));
}
