use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use bioforge_core::analysis;
use crate::jit::ValorizationRequest;

mod config;
mod jit;
mod plotting;
mod workflow;

fn main() -> Result<()> {
    println!("--- Bioforge Application ---");

    // --- Target Selection ---
    // Load the request from the YAML file
    let request_str = fs::read_to_string("bioforge-app/request.yaml")
        .context("Failed to read request.yaml")?;
    let request: ValorizationRequest = serde_yaml::from_str(&request_str)
        .context("Failed to parse request.yaml")?;

    let kb = config::KnowledgeBase::load("./data/knowledge_base")?;

    let upstream_organisms = jit::select_optimal_organism_mix(&request, &kb)?;
    let downstream_processes = jit::select_downstream_processes(&request, &kb)?;

    let output_dir = format!("./data/runs/Lutein_bGlucan_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir))?;

    // Copy the request file to the output directory for traceability
    fs::copy("bioforge-app/request.yaml", Path::new(&output_dir).join("request.yaml"))?;


    // Generate the initial media for the selected organisms
    let initial_media = jit::generate_initial_media(&upstream_organisms, &output_dir)?;
    
    // Create a BOM for the initial media
    let initial_bom = analysis::bom_from_media_state(&initial_media)?;

    let upstream_output = workflow::run_upstream_simulations(&upstream_organisms, &kb, &output_dir, initial_media, &request)?;
    
    workflow::run_downstream_and_report(&downstream_processes, &upstream_output, &kb, &output_dir, &request, &upstream_organisms, initial_bom)?;

    println!("\nEnd-to-end workflow complete. Results are in '{}'", output_dir);

    Ok(())
}