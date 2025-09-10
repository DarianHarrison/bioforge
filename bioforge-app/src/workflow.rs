use crate::config::KnowledgeBase;
use crate::jit;
use crate::plotting;
use anyhow::Result;
use bioforge_core::{
    analysis::{self, BillOfMaterials},
    simulation::builder::SimulationBuilder,
};
use bioforge_schemas::{
    command::Command,
    environment::MediaState,
    organism::Organism,
    process::{Method, Process},
    rule::{Condition, Rule},
    rule::ComparisonOperator,
};
use std::{collections::HashMap, fs, path::Path};

/// Represents the output of the combined upstream simulations.
#[derive(Debug, Clone)]
pub struct UpstreamOutput {
    pub biomass_produced: HashMap<String, f64>,
    pub combined_bom: BillOfMaterials,
}

/// Orchestrates a single upstream cultivation simulation for the selected consortium of organisms.
pub fn run_upstream_simulations(
    organisms: &[Organism],
    kb: &KnowledgeBase,
    output_dir: &str,
    initial_media: MediaState,
    request: &jit::ValorizationRequest,
) -> Result<UpstreamOutput> {
    println!("\n--- [Workflow] Starting Upstream Consortium Simulation ---");

    let organism_clones: Vec<Organism> = organisms.to_vec();
    let organism_names: HashMap<String, String> = organism_clones
        .iter()
        .map(|o| (o.organism_id.clone(), o.organism_name.clone()))
        .collect();
    
    let log_path = Path::new(output_dir).join("upstream_consortium.csv");

    let mut rules = Vec::new();

    // Rule to stop the entire simulation when the slowest target is met
    let lutein_target = request.targets.iter().find(|t| t.molecule_name == "Lutein").unwrap();
    rules.push(Rule {
        name: "rule_stop_on_lutein".to_string(),
        condition: Condition::ProductAmount {
            molecule_name: lutein_target.molecule_name.clone(),
            target_grams: lutein_target.target_amount_grams,
        },
        action: Command::AdvanceToNextStep,
    });
    
    // Rule to stop the growth of the faster organism when its target is met
    let beta_glucan_target = request.targets.iter().find(|t| t.molecule_name == "beta-glucans").unwrap();
    let agrobacterium_id = "ORG-AGROSP";
    rules.push(Rule {
        name: "rule_stop_agrobacterium_growth".to_string(),
        condition: Condition::ProductAmount {
            molecule_name: beta_glucan_target.molecule_name.clone(),
            target_grams: beta_glucan_target.target_amount_grams,
        },
        action: Command::SetOrganismGrowthMultiplier {
            organism_id: agrobacterium_id.to_string(),
            multiplier: 0.0,
        },
    });


    let feed_rule = Rule {
        name: "rule_feed_sucrose".to_string(),
        condition: Condition::MediaValue {
            molecule_id: "CHEBI:17992".to_string(), // Correctly targeting Sucrose now
            operator: ComparisonOperator::LessThan,
            value: 1.0, // g/L
        },
        action: Command::AddMaterial {
            asset_id: "CULTIVATION-LOOP-01".to_string(),
            material_id: "CHEBI:17992".to_string(),
            amount_grams: 2500.0, // Increased amount for a visible spike
        },
    };
    rules.push(feed_rule);

    let cultivation_method = Method {
        method_id: "MTHD-UP-CULT-DYNAMIC-01".to_string(),
        stage: "Cultivation".to_string(),
        technique: "fed-batch".to_string(),
        required_asset_id: "CULTIVATION-LOOP-01".to_string(),
        operating_parameters: HashMap::new(),
        required_materials: vec![],
        qc_checks: vec![],
        required_rule_ids: Some(rules.iter().map(|r| r.name.clone()).collect()),
    };

    let upstream_process = Process {
        process_id: "PROC-UPSTREAM-CULTIVATION-DYNAMIC".to_string(),
        process_name: "Dynamic Upstream Cultivation".to_string(),
        component_class: "Cultivation".to_string(),
        status: "Active".to_string(),
        notes: "A dynamically generated, single-stage cultivation process.".to_string(),
        default_workflow: vec![cultivation_method.method_id.clone()],
        methods: vec![cultivation_method],
    };

    let mut sim_rules = kb.rules.clone();
    for rule in rules {
        sim_rules.insert(rule.name.clone(), rule);
    }

    let mut engine = SimulationBuilder::new()
        .with_organisms(organism_clones)
        .with_assets(kb.assets.values().cloned().collect())
        .with_rules(sim_rules.values().cloned().collect())
        .with_process(upstream_process)
        .with_initial_media(initial_media)
        .with_timeseries_logging_to_file(log_path.to_str().unwrap())
        .build()?;

    engine.run()?;
    
    let final_biomass_states = engine.get_organism_states();
    let biomass_produced = final_biomass_states
        .iter()
        .map(|(id, state)| (id.clone(), state.biomass.value))
        .collect::<HashMap<_,_>>();

    let bom = analysis::generate_bom(log_path.to_str().unwrap(), engine.get_process(), &kb.assets, &kb.materials)?;

    let placeholder_cogs = analysis::CogsResult::default();
    let placeholder_lca = analysis::LcaResult::default();
    plotting::generate_all_plots(output_dir, log_path.to_str().unwrap(), &placeholder_cogs, &placeholder_lca, organism_names)?;

    Ok(UpstreamOutput {
        biomass_produced,
        combined_bom: bom,
    })
}


/// Orchestrates the downstream processing simulations and generates the final reports.
pub fn run_downstream_and_report(
    processes: &[&Process],
    upstream_output: &UpstreamOutput,
    kb: &KnowledgeBase,
    output_dir: &str,
    request: &jit::ValorizationRequest,
    upstream_organisms: &[Organism],
    initial_bom: BillOfMaterials,
) -> Result<()> {
    println!("\n--- [Workflow] Starting Downstream Simulations ---");
    let mut all_boms = vec![initial_bom, upstream_output.combined_bom.clone()];

    for process in processes {
        println!("\nProcessing for: {}", process.process_name);
        let log_path =
            Path::new(output_dir).join(format!("downstream_{}.csv", process.process_id));

        let placeholder_org = kb.organisms.values().next().unwrap().clone();

        let initial_media = jit::generate_initial_media(&[placeholder_org.clone()], output_dir)?;

        let mut engine = SimulationBuilder::new()
            .with_organisms(vec![placeholder_org])
            .with_assets(kb.assets.values().cloned().collect())
            .with_rules(kb.rules.values().cloned().collect())
            .with_process((*process).clone())
            .with_initial_media(initial_media)
            .with_timeseries_logging_to_file(log_path.to_str().unwrap())
            .build()?;

        engine.run()?;

        let bom =
            analysis::generate_bom(log_path.to_str().unwrap(), process, &kb.assets, &kb.materials)?;
        all_boms.push(bom);
    }

    println!("\n--- [Workflow] Aggregating Reports ---");
    let final_bom = aggregate_boms(all_boms);

    let final_cogs = analysis::calculate_cogs(&final_bom, &kb.materials, &kb.labor_roles, &kb.assets)?;
    let final_lca = analysis::calculate_lca(&final_bom, &kb.materials, &kb.assets)?;

    let qca_table = generate_qca_table(processes);
    fs::write(Path::new(output_dir).join("qca_report.md"), qca_table)?;

    plotting::plot_process_flow(output_dir, processes, &kb.rules)?;

    print_summary_report(&final_bom, &final_cogs, &final_lca, processes, request, upstream_output, kb, upstream_organisms);

    Ok(())
}

fn aggregate_boms(boms: Vec<BillOfMaterials>) -> BillOfMaterials {
    let mut combined_bom = BillOfMaterials::default();
    for bom in boms {
        combined_bom.total_energy_kwh += bom.total_energy_kwh;
        combined_bom.total_ticks += bom.total_ticks;
        for (material, quantity) in bom.materials_consumed {
            *combined_bom
                .materials_consumed
                .entry(material)
                .or_insert(0.0) += quantity;
        }
        for (role, hours) in bom.labor_hours {
            *combined_bom.labor_hours.entry(role).or_insert(0.0) += hours;
        }
    }
    combined_bom
}

fn generate_qca_table(processes: &[&Process]) -> String {
    let mut table = String::from("| Process Stage | QC Method ID | Timing |\n");
    table.push_str("|---------------|--------------|----------|\n");

    for process in processes {
        for method in &process.methods {
            if method.qc_checks.is_empty() {
                table.push_str(&format!("| {} | *None* | N/A |\n", method.stage));
            }
            for qc in &method.qc_checks {
                table.push_str(&format!(
                    "| {} | {} | {} |\n",
                    method.stage, qc.method_id, qc.timing
                ));
            }
        }
    }
    table
}

fn print_summary_report(
    bom: &analysis::BillOfMaterials,
    cogs: &analysis::CogsResult,
    lca: &analysis::LcaResult,
    processes: &[&Process],
    request: &jit::ValorizationRequest,
    upstream_output: &UpstreamOutput,
    kb: &KnowledgeBase,
    upstream_organisms: &[Organism],
) {
    let process_names: Vec<&str> = processes.iter().map(|p| p.process_name.as_str()).collect();
    
    println!("\n\n--- [Final Summary Report] ---");
    println!("========================================");
    println!("Request & Production Summary:");
    
    for target in &request.targets {
        let mut produced_grams = 0.0;
        // Find which of the selected organisms produces the target molecule
        if let Some(producing_organism) = upstream_organisms.iter().find(|org| jit::find_yield(org, &target.molecule_name).is_some()) {
            if let Some(biomass) = upstream_output.biomass_produced.get(&producing_organism.organism_id) {
                if let Some(yield_mg_g) = jit::find_yield(producing_organism, &target.molecule_name) {
                    produced_grams = biomass * yield_mg_g / 1000.0; // Convert mg to g
                }
            }
        }

        println!(
            "  - Target: {:<12} | Produced: {:>8.2} g / Requested: {:>8.2} g ({:.1}% of target)",
            target.molecule_name,
            produced_grams,
            target.target_amount_grams,
            (produced_grams / target.target_amount_grams) * 100.0
        );
    }
    
    println!("\nProcesses Used: {}", process_names.join(", "));
    println!("Simulation Duration: {} hours", bom.total_ticks);
    println!("----------------------------------------");

    println!("\nCombined Bill of Materials (BOM):");
    println!("  - Energy Consumed: {:.2} kWh", bom.total_energy_kwh);
    println!("  - Materials Consumed:");
    for (id, qty) in &bom.materials_consumed {
        let material_name = kb.materials.get(id).map_or(id.as_str(), |m| m.material_name.as_str());
        println!("    - {}: {:.4} kg", material_name, qty / 1000.0); // Convert grams to kg
    }

    println!("\nCombined Cost of Goods Sold (COGS):");
    println!("  - Material Costs:           ${:.2} USD", cogs.material_costs);
    println!("  - Labor Costs:              ${:.2} USD", cogs.labor_costs);
    println!("  - Energy Costs:             ${:.2} USD", cogs.energy_costs);
    println!("  - Asset Depreciation:       ${:.2} USD", cogs.asset_depreciation_costs);
    println!("  - Maintenance Costs:        ${:.2} USD", cogs.maintenance_costs);
    println!("  --------------------------------------");
    println!("  - Total COGS:               ${:.2} USD", cogs.total_cogs);

    println!("\nCombined Life Cycle Assessment (LCA):");
    println!(
        "  - Global Warming Potential: {:.2} kg CO₂e",
        lca.gwp_kg_co2e
    );
    println!(
        "  - Abiotic Depletion (fossil): {:.2} MJ",
        lca.adp_fossil_mj
    );

    println!("========================================");
}