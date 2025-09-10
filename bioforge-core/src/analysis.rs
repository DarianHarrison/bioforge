use bioforge_schemas::{
    asset::Asset,
    environment::MediaState,
    labor::LaborRole,
    material::{Material},
    process::Process,
    rule::Rule,
};
use crate::{
    error::BioforgeError,
    simulation::state::SimulationEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct LogEntry {
    pub tick: u64,
    pub stage_id: String,
    pub organisms_json: String,
    pub media_volume_l: f64,
    pub media_ph: f64,
    pub dissolved_components_json: String,
    pub dissolved_gases_json: String,
    pub asset_states_json: String,
    pub events_json: String,
}


#[derive(Debug, Default, Clone)]
pub struct BillOfMaterials {
    pub materials_consumed: HashMap<String, f64>,
    pub total_energy_kwh: f64,
    pub labor_hours: HashMap<String, f64>,
    pub total_ticks: u64,
}

#[derive(Debug, Default, Clone)]
pub struct CogsResult {
    pub material_costs: f64,
    pub labor_costs: f64,
    pub energy_costs: f64,
    pub asset_depreciation_costs: f64,
    pub maintenance_costs: f64,
    pub total_cogs: f64,
}

#[derive(Debug, Default, Clone)]
pub struct LcaResult {
    pub gwp_kg_co2e: f64,
    pub adp_fossil_mj: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlueprintStep {
    pub step: usize,
    pub method_id: String,
    pub technique: String,
    pub asset_id: String,
    pub duration_ticks: u64,
    pub control_parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutableBlueprint {
    pub process_id: String,
    pub process_name: String,
    pub workflow: Vec<BlueprintStep>,
}

pub fn bom_from_media_state(
    media_state: &MediaState
) -> Result<BillOfMaterials, BioforgeError> {
    let mut bom = BillOfMaterials::default();
    let media_volume = media_state.volume.value;

    for component in &media_state.composition.dissolved_components {
        let total_grams = component.concentration.value * media_volume;
        *bom.materials_consumed.entry(component.molecule_id.clone()).or_insert(0.0) += total_grams;
    }
    Ok(bom)
}

pub fn generate_bom(
    log_path: &str,
    process: &Process,
    assets: &HashMap<String, Asset>,
    materials: &HashMap<String, Material>,
) -> Result<BillOfMaterials, BioforgeError> {
    let mut reader = csv::Reader::from_path(log_path)
        .map_err(|e| BioforgeError::CsvError(log_path.to_string(), e))?;
    let mut bom = BillOfMaterials::default();
    let mut ticks_in_stage: HashMap<String, u64> = HashMap::new();

    for result in reader.deserialize() {
        let record: LogEntry =
            result.map_err(|e| BioforgeError::CsvError(log_path.to_string(), e))?;
        *ticks_in_stage.entry(record.stage_id.clone()).or_insert(0) += 1;
        bom.total_ticks +=1;

        let events: Vec<SimulationEvent> = serde_json::from_str(&record.events_json)?;
        for event in events {
            match event {
                SimulationEvent::MaterialConsumed { id, amount } => {
                    if let Some(material) = materials.get(&id) {
                        *bom.materials_consumed.entry(material.material_id.clone()).or_insert(0.0) += amount;
                    } else if let Some(material) = materials.values().find(|m| m.metadata.identifiers.as_ref().map_or(false, |i| i.chebi_id == Some(id.clone()))) {
                        *bom.materials_consumed.entry(material.material_id.clone()).or_insert(0.0) += amount;
                    }
                }
                SimulationEvent::MaterialAdded { .. } => {
                    // Not currently tracking added materials in the BOM
                }
            }
        }

        if let Some(method) = process.methods.iter().find(|m| m.method_id == record.stage_id) {
            if let Some(asset) = assets.get(&method.required_asset_id) {
                if let Some(params) = &asset.operational_parameters {
                    if let Some(power_model) = &params.power_model {
                        bom.total_energy_kwh += power_model.operating_power.value;
                    }
                }
            }
        }
    }
    
    for (stage_id, total_ticks) in ticks_in_stage {
         if let Some(method) = process.methods.iter().find(|m| m.method_id == stage_id) {
            if let Some(asset) = assets.get(&method.required_asset_id) {
                if let Some(params) = &asset.operational_parameters {
                    if let Some(labor_reqs) = &params.labor_requirements {
                        for req in labor_reqs {
                            let hours = match req.duration.unit.as_str() {
                                "min" => req.duration.value / 60.0,
                                "min/hr_op" => (req.duration.value / 60.0) * total_ticks as f64,
                                "min/box" => req.duration.value / 60.0, // Assuming 1 box op
                                "min/10L" => (req.duration.value / 60.0) * (bom.total_ticks as f64 / 10.0), // Example logic
                                _ => req.duration.value, // Assume hours if not specified
                            };
                            *bom.labor_hours.entry(req.required_role_id.clone()).or_insert(0.0) += hours;
                        }
                    }
                }
            }
        }
    }

    Ok(bom)
}

pub fn calculate_cogs(
    bom: &BillOfMaterials,
    materials: &HashMap<String, Material>,
    labor_roles: &HashMap<String, LaborRole>,
    assets: &HashMap<String, Asset>,
) -> Result<CogsResult, BioforgeError> {
    let mut result = CogsResult::default();
    let cost_per_kwh = 0.12;
    let hours_per_year = 8760.0;
    let simulation_duration_hours = bom.total_ticks as f64;

    for (material_id, quantity) in &bom.materials_consumed {
        let material_to_cost = materials.values().find(|m| m.metadata.identifiers.as_ref().map_or(false, |i| i.chebi_id == Some(material_id.clone())));
        if let Some(material) = material_to_cost {
            let cost_per_unit = material.techno_economic_and_lca_profile.lifecycle_stages.manufacturing_and_acquisition.costs.get(0).map_or(0.0, |c| c.value_usd);
            let total_cost = (quantity / 1000.0) * cost_per_unit;
            result.material_costs += total_cost;
        }
    }

    for (role_id, hours) in &bom.labor_hours {
        if let Some(role) = labor_roles.get(role_id) {
            result.labor_costs += hours * role.techno_economic_profile.cost_per_hour_usd;
        }
    }

    for asset in assets.values() {
        if let Some(tea) = &asset.techno_economic_and_lca_profile {
            let lifespan_years = tea.expected_lifespan.as_ref().map_or(1, |l| l.value) as f64;
            if let Some(capex) = tea.lifecycle_stages.manufacturing_and_acquisition.costs.iter().find(|c| c.cost_type == "capex") {
                let annual_depreciation = capex.value_usd / lifespan_years;
                result.asset_depreciation_costs += (annual_depreciation / hours_per_year) * simulation_duration_hours;
            }
            if let Some(maintenance_cost) = tea.lifecycle_stages.maintenance.costs.iter().find(|c| c.cost_type == "opex_per_year") {
                result.maintenance_costs += (maintenance_cost.value_usd / hours_per_year) * simulation_duration_hours;
            }
        }
    }

    result.energy_costs = bom.total_energy_kwh * cost_per_kwh;
    result.total_cogs = result.material_costs + result.labor_costs + result.energy_costs + result.asset_depreciation_costs + result.maintenance_costs;

    Ok(result)
}

pub fn calculate_lca(
    bom: &BillOfMaterials,
    materials: &HashMap<String, Material>,
    assets: &HashMap<String, Asset>,
) -> Result<LcaResult, BioforgeError> {
    let mut result = LcaResult::default();
    let gwp_per_kwh = 0.4;
    let adp_fossil_per_kwh = 8.0;
    let hours_per_year = 8760.0;
    let simulation_duration_hours = bom.total_ticks as f64;

    for (material_id, quantity) in &bom.materials_consumed {
        if let Some(material) = materials.get(material_id) {
            let impacts = &material.techno_economic_and_lca_profile.lifecycle_stages.manufacturing_and_acquisition.impacts;
            if let Some(gwp) = impacts.iter().find(|i| i.metric == "gwp") {
                result.gwp_kg_co2e += quantity * gwp.value;
            }
            if let Some(adp) = impacts.iter().find(|i| i.metric == "adp_fossil") {
                result.adp_fossil_mj += quantity * adp.value;
            }
        }
    }

    for asset in assets.values() {
        if let Some(tea) = &asset.techno_economic_and_lca_profile {
            if let Some(gwp) = tea.lifecycle_stages.use_and_operation.impacts.iter().find(|i| i.metric == "gwp_per_year") {
                result.gwp_kg_co2e += (gwp.value / hours_per_year) * simulation_duration_hours;
            }
            if let Some(adp) = tea.lifecycle_stages.use_and_operation.impacts.iter().find(|i| i.metric == "adp_fossil_per_year") {
                result.adp_fossil_mj += (adp.value / hours_per_year) * simulation_duration_hours;
            }
        }
    }

    result.gwp_kg_co2e += bom.total_energy_kwh * gwp_per_kwh;
    result.adp_fossil_mj += bom.total_energy_kwh * adp_fossil_per_kwh;

    Ok(result)
}


pub fn generate_blueprint(
    process: &Process,
    rules: &HashMap<String, Rule>,
) -> Result<ExecutableBlueprint, BioforgeError> {
    let mut workflow = Vec::new();

    for (i, method_id) in process.default_workflow.iter().enumerate() {
        let method = process
            .methods
            .iter()
            .find(|m| m.method_id == *method_id)
            .ok_or_else(|| BioforgeError::MethodNotFound(method_id.clone()))?;

        let duration_rule_id = method
            .required_rule_ids
            .as_ref()
            .and_then(|ids| {
                ids.iter().find(|id| {
                    rules
                        .get(*id)
                        .map_or(false, |r| matches!(r.condition, bioforge_schemas::rule::Condition::TimeInStage { .. }))
                })
            })
            .ok_or_else(|| BioforgeError::ConfigError(format!("Could not find a duration rule for method '{}'", method_id)))?;

        let duration_ticks = if let Some(rule) = rules.get(duration_rule_id) {
            if let bioforge_schemas::rule::Condition::TimeInStage { ticks } = rule.condition {
                ticks
            } else {
                0
            }
        } else {
            0
        };

        let step = BlueprintStep {
            step: i + 1,
            method_id: method.method_id.clone(),
            technique: method.technique.clone(),
            asset_id: method.required_asset_id.clone(),
            duration_ticks,
            control_parameters: method.operating_parameters.clone(),
        };
        workflow.push(step);
    }

    Ok(ExecutableBlueprint {
        process_id: process.process_id.clone(),
        process_name: process.process_name.clone(),
        workflow,
    })
}