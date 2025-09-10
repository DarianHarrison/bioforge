use crate::config::KnowledgeBase;
use anyhow::{Context, Result};
use bioforge_schemas::{
    environment::{DissolvedComponent, MediaComposition, MediaState, Measurement},
    organism::Organism,
    process::Process,
};
use std::{collections::HashMap, fs, path::Path};
use serde::Deserialize;


/// Represents a high-level goal for the bioprocess, now supporting multiple targets.
#[derive(Debug, Deserialize)]
pub struct ValorizationRequest {
    pub targets: Vec<TargetRequest>,
}

/// Defines a specific target molecule and the objective for its production.
#[derive(Debug, Deserialize)]
pub struct TargetRequest {
    pub molecule_name: String,
    pub objective: Objective,
    pub process_id: String, // Explicitly define the downstream process
    pub target_amount_grams: f64, // The desired final amount of the molecule
}

#[derive(Debug, Deserialize)]
pub enum Objective {
    MaximizeYield,
    MinimizeCost,
    MinimizeLca,
}

/// JIT Optimizer: selects the best set of organisms to fulfill the multi-target request.
pub fn select_optimal_organism_mix(
    request: &ValorizationRequest,
    kb: &KnowledgeBase,
) -> Result<Vec<Organism>> {
    println!("\n--- [JIT] Running Upstream Optimizer ---");
    let mut organism_map: HashMap<String, Organism> = HashMap::new();

    // First, select the best organism for each target and store a clone
    for target in &request.targets {
        println!("Optimizing for target: {}", target.molecule_name);
        let best_organism = match target.objective {
            Objective::MaximizeYield => kb
                .organisms
                .values()
                .filter_map(|org| {
                    let yield_value = find_yield(org, &target.molecule_name);
                    yield_value.map(|y| (org, y))
                })
                .max_by(|(_, yield_a), (_, yield_b)| {
                    yield_a
                        .partial_cmp(yield_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(org, _)| org)
                .context(format!(
                    "Optimizer failed to find an organism for '{}'",
                    target.molecule_name
                ))?,
            Objective::MinimizeCost | Objective::MinimizeLca => {
                println!("Warning: MinimizeCost/MinimizeLca not yet implemented. Defaulting to MaximizeYield.");
                kb.organisms
                    .values()
                    .filter_map(|org| {
                        let yield_value = find_yield(org, &target.molecule_name);
                        yield_value.map(|y| (org, y))
                    })
                    .max_by(|(_, yield_a), (_, yield_b)| {
                        yield_a
                            .partial_cmp(yield_b)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(org, _)| org)
                    .context(format!(
                        "Optimizer failed to find an organism for '{}'",
                        target.molecule_name
                    ))?
            }
        };

        if !organism_map.contains_key(&best_organism.organism_id) {
            organism_map.insert(best_organism.organism_id.clone(), best_organism.clone());
        }
    }

    // Now, calculate the required biomass for each selected organism
    let mut required_biomasses: HashMap<String, f64> = HashMap::new();
    for org in organism_map.values() {
        if let Some(target) = request.targets.iter().find(|t| find_yield(org, &t.molecule_name).is_some()) {
            if let Some(yield_mg_g) = find_yield(org, &target.molecule_name) {
                if yield_mg_g > 0.0 {
                    let required = target.target_amount_grams / (yield_mg_g / 1000.0);
                    required_biomasses.insert(org.organism_id.clone(), required);
                }
            }
        }
    }
    
    // Find the maximum required biomass to use as a scaling reference
    let max_required_biomass = required_biomasses.values().cloned().fold(0.0, f64::max);
    
    if max_required_biomass > 0.0 {
        println!("\n--- [JIT] Adjusting Initial Inoculum Ratios ---");
        let default_initial_biomass = 0.1; // Base inoculum size in grams

        // Adjust the initial_biomass for each organism in our map
        for (org_id, org) in organism_map.iter_mut() {
            if let Some(required) = required_biomasses.get(org_id) {
                let scaled_initial_biomass = (required / max_required_biomass) * default_initial_biomass;
                println!(
                    "Adjusting initial biomass for {} from {}g to {:.4}g (target: {:.2}g)", 
                    org_id, org.initial_biomass.value, scaled_initial_biomass, required
                );
                org.initial_biomass.value = scaled_initial_biomass.max(1e-6); // Prevent zero/negative biomass
            }
        }
    }

    let selected_organisms: Vec<Organism> = organism_map.into_values().collect();
    
    println!(
        "Final organism set selected: {:?}",
        selected_organisms
            .iter()
            .map(|o| &o.organism_id)
            .collect::<Vec<_>>()
    );
    Ok(selected_organisms)
}


/// Dynamically generates the initial media formulation based on the metabolic needs of the selected organisms.
pub fn generate_initial_media(
    organisms: &[Organism],
    output_dir: &str,
) -> Result<MediaState> {
    println!("\n--- [JIT] Generating Initial Media Formulation ---");
    let mut dissolved_components = HashMap::new();

    // Add common base components
    dissolved_components.insert(
        "CHEBI:132204".to_string(), // ammonia
        DissolvedComponent {
            molecule_id: "CHEBI:132204".to_string(),
            molecule_name: "ammonia".to_string(),
            concentration: Measurement { value: 2.0, unit: "g/L".to_string() },
        },
    );

    // Add carbon sources required by the selected organisms
    for org in organisms {
        for consumption in &org.dynamic_parameters.metabolic_exchange.media_consumption {
            if !dissolved_components.contains_key(&consumption.molecule_id) {
                println!("Adding required nutrient: {}", consumption.molecule_name);
                dissolved_components.insert(
                    consumption.molecule_id.clone(),
                    DissolvedComponent {
                        molecule_id: consumption.molecule_id.clone(),
                        molecule_name: consumption.molecule_name.clone(),
                        concentration: Measurement { value: 20.0, unit: "g/L".to_string() }, // Default concentration
                    },
                );
            }
        }
    }

    let media_state = MediaState {
        volume: Measurement { value: 500.0, unit: "L".to_string() },
        ph: 7.0,
        composition: MediaComposition {
            dissolved_components: dissolved_components.values().cloned().collect(),
            dissolved_gases: vec![
                bioforge_schemas::environment::DissolvedGas {
                    gas_id: "CHEBI:15379".to_string(),
                    gas_name: "oxygen".to_string(),
                    concentration: Measurement { value: 0.008, unit: "g/L".to_string() },
                }
            ],
        },
    };

    let media_path = Path::new(output_dir).join("initial_media.yaml");
    let yaml_content = serde_yaml::to_string(&media_state)?;
    fs::write(media_path, yaml_content)?;

    Ok(media_state)
}

/// JIT Optimizer: selects the best downstream process for each target.
pub fn select_downstream_processes<'a>(
    request: &ValorizationRequest,
    kb: &'a KnowledgeBase,
) -> Result<Vec<&'a Process>> {
    println!("\n--- [JIT] Running Downstream Optimizer ---");
    let mut selected_processes = Vec::new();

    for target in &request.targets {
        let best_process = kb
            .processes
            .get(&target.process_id)
            .context(format!(
                "Optimizer failed to find a downstream process with id '{}'",
                target.process_id
            ))?;
        
        selected_processes.push(best_process);
        println!("Selected process '{}' for target '{}'", best_process.process_id, target.molecule_name);
    }
    Ok(selected_processes)
}

/// Helper function to find the yield of a specific molecule in an organism.
pub fn find_yield(organism: &Organism, molecule_name: &str) -> Option<f64> {
    organism
        .static_properties
        .targeted_molecular_classes
        .terpenoids_and_carotenoids
        .iter()
        .find(|m| m.molecule == molecule_name)
        .map(|m| m.concentration_mg_g_dw)
        .or_else(|| {
            organism
                .static_properties
                .targeted_molecular_classes
                .cell_wall_components
                .iter()
                .find(|m| m.molecule == molecule_name)
                .map(|m| m.concentration_mg_g_dw)
        })
}