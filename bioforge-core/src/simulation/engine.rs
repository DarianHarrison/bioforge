use super::{
    state::{LiveAsset, SimulationEvent, SimulationState},
};
use crate::{error::BioforgeError, logger::TimeSeriesLogger};
use bioforge_schemas::{
    command::Command,
    environment::{DissolvedComponent, MediaState, Measurement},
    organism::Organism,
    organism_state::IndividualOrganismState,
    process::Process,
    rule::{ComparisonOperator, Condition, Rule},
};
use std::collections::{HashMap, VecDeque};

pub struct SimulationEngine {
    pub(super) state: SimulationState,
    pub(super) process: Process,
    pub(super) rules: HashMap<String, Rule>,
    pub(super) organism_defs: HashMap<String, Organism>,
    pub(super) current_step_index: usize,
    pub(super) logger: Option<TimeSeriesLogger>,
    pub(super) biomass_history: VecDeque<f64>,
    pub(super) growth_multipliers: HashMap<String, f64>,
}

impl SimulationEngine {
    pub fn run(&mut self) -> Result<(), BioforgeError> {
        if let Some(initial_method_id) = self.process.default_workflow.get(self.current_step_index) {
            println!("--- Entering stage: {} ---", initial_method_id);
        }

        if let Some(logger) = &mut self.logger {
            logger.log_state(&self.state, "INITIAL")?;
        }

        loop {
            // The tick method will return false when the simulation is complete
            if !self.tick()? {
                break;
            }
        }
        println!("Simulation Complete.");
        Ok(())
    }

    pub fn tick(&mut self) -> Result<bool, BioforgeError> {
        if self.current_step_index >= self.process.default_workflow.len() {
            return Ok(false);
        }

        self.state.events.clear();
        self.state.tick += 1;
        self.state.ticks_in_current_stage += 1;

        self.execute_biological_tick()?;
        self.execute_unit_operation_tick()?;

        let current_method_id = self.process.default_workflow[self.current_step_index].clone();
        let current_method = self
            .process
            .methods
            .iter()
            .find(|m| m.method_id == current_method_id)
            .ok_or_else(|| BioforgeError::MethodNotFound(current_method_id.clone()))?;

        let mut command_queue: Vec<Command> = Vec::new();
        if let Some(rule_ids) = &current_method.required_rule_ids {
            for rule_id in rule_ids {
                if let Some(rule) = self.rules.get(rule_id) {
                    if self.evaluate_condition(&rule.condition)? {
                        command_queue.push(rule.action.clone());
                    }
                }
            }
        }

        if let Some(logger) = &mut self.logger {
            logger.log_state(&self.state, &current_method_id)?;
        }

        for command in command_queue {
            self.execute_command(command)?;
        }

        Ok(true)
    }

    fn execute_unit_operation_tick(&mut self) -> Result<(), BioforgeError> {
        let current_method = &self.process.methods[self.current_step_index];

        match current_method.technique.as_str() {
            "saponification" => {
                let naoh_id = "CHEBI:32145";
                let consumption_rate = 0.5_f64;

                if let Some(naoh) = self
                    .state
                    .media
                    .composition
                    .dissolved_components
                    .iter_mut()
                    .find(|c| c.molecule_id == naoh_id)
                {
                    if naoh.concentration.value > 0.0 {
                        let consumed_conc = consumption_rate.min(naoh.concentration.value);
                        naoh.concentration.value -= consumed_conc;

                        let consumed_amount_g = consumed_conc * self.state.media.volume.value;
                        self.state.events.push(SimulationEvent::MaterialConsumed {
                            id: "CONS-NAOH-1M-01".to_string(),
                            amount: consumed_amount_g,
                        });
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_biological_tick(&mut self) -> Result<(), BioforgeError> {
        let time_step_hr = 1.0;
        let mut media_deltas: HashMap<String, f64> = HashMap::new();
        let mut new_byproducts: Vec<DissolvedComponent> = Vec::new();
        let mut total_biomass_this_tick = 0.0;

        for (org_id, org_state) in self.state.organisms.states.iter_mut() {
            let org_def = self.organism_defs.get(org_id).ok_or_else(|| BioforgeError::OrganismNotFound(org_id.clone()))?;
            let bioreactor_id = &self.process.methods[self.current_step_index].required_asset_id;
            let asset = self.state.assets.get(bioreactor_id);
            let bioreactor_temp = asset.map_or(
                org_def.dynamic_parameters.environmental_tolerances.temperature.optimal.value,
                |a| a.temperature,
            );

            let temp_tolerance = &org_def.dynamic_parameters.environmental_tolerances.temperature;
            let stress_factor = if bioreactor_temp < temp_tolerance.range.min || bioreactor_temp > temp_tolerance.range.max {
                0.1
            } else if bioreactor_temp <= temp_tolerance.optimal.value {
                0.1 + 0.9 * (bioreactor_temp - temp_tolerance.range.min) / (temp_tolerance.optimal.value - temp_tolerance.range.min)
            } else {
                1.0 - 0.9 * (bioreactor_temp - temp_tolerance.optimal.value) / (temp_tolerance.range.max - temp_tolerance.optimal.value)
            };
            
            let k_s = 0.5; 
            let primary_carbon_source_name = org_def.dynamic_parameters.metabolic_exchange.media_consumption.get(0).map_or("", |c| &c.molecule_name);
            let nutrient_concentration = self.state.media.composition.dissolved_components.iter()
                .find(|c| c.molecule_name == primary_carbon_source_name)
                .map_or(0.0, |c| c.concentration.value);
            
            let nutrient_limitation_factor = nutrient_concentration / (k_s + nutrient_concentration);
            
            let growth_multiplier = *self.growth_multipliers.get(org_id).unwrap_or(&1.0);
            let growth_rate = org_def.dynamic_parameters.growth_rate_per_hr * stress_factor * nutrient_limitation_factor * growth_multiplier;
            let growth = org_state.biomass.value * ((growth_rate * time_step_hr).exp() - 1.0);
            org_state.biomass.value += growth;

            total_biomass_this_tick += org_state.biomass.value;

            for consumption_def in &org_def.dynamic_parameters.metabolic_exchange.media_consumption {
                if let Some(nutrient) = self.state.media.composition.dissolved_components.iter().find(|c| c.molecule_id == consumption_def.molecule_id) {
                    if nutrient.concentration.value > 0.0 {
                        let nutrient_mw = if consumption_def.molecule_id == "CHEBI:17234" { 180.16 } else { 342.3 };
                        
                        let consumption_rate_g_gdw_hr = consumption_def.max_exchange_rate.value * nutrient_mw / 1000.0 * growth_multiplier;
                        let max_consumption_g = consumption_rate_g_gdw_hr * org_state.biomass.value * time_step_hr;
                        let available_nutrient_g = nutrient.concentration.value * self.state.media.volume.value;
                        let actual_consumption_g = max_consumption_g.min(available_nutrient_g);

                        if actual_consumption_g > 0.0 {
                            let delta_conc = actual_consumption_g / self.state.media.volume.value;
                            *media_deltas.entry(consumption_def.molecule_id.clone()).or_insert(0.0) -= delta_conc;

                            self.state.events.push(SimulationEvent::MaterialConsumed {
                                id: consumption_def.molecule_id.clone(),
                                amount: actual_consumption_g,
                            });
                        }
                    }
                }
            }

            for secretion_def in &org_def.dynamic_parameters.metabolic_exchange.media_secretion {
                let byproduct_mw = if secretion_def.molecule_id == "CHEBI:30089" { 60.05 } else { 1.0 }; 
                let secretion_rate_g_gdw_hr = secretion_def.max_exchange_rate.value * byproduct_mw / 1000.0;
                let secreted_amount_g = secretion_rate_g_gdw_hr * org_state.biomass.value * time_step_hr * stress_factor;

                if secreted_amount_g > 0.0 {
                    let delta_conc = secreted_amount_g / self.state.media.volume.value;
                    *media_deltas.entry(secretion_def.molecule_id.clone()).or_insert(0.0) += delta_conc;

                    if self.state.media.composition.dissolved_components.iter().find(|c| c.molecule_id == secretion_def.molecule_id).is_none() {
                        if !new_byproducts.iter().any(|b| b.molecule_id == secretion_def.molecule_id) {
                            new_byproducts.push(DissolvedComponent {
                                molecule_id: secretion_def.molecule_id.clone(),
                                molecule_name: secretion_def.molecule_name.clone(),
                                concentration: Measurement { value: 0.0, unit: "g/L".to_string() },
                            });
                        }
                    }
                }
            }
        }

        self.biomass_history.push_back(total_biomass_this_tick);
        if self.biomass_history.len() > 10 {
            self.biomass_history.pop_front();
        }

        self.state.media.composition.dissolved_components.extend(new_byproducts);

        for (molecule_id, delta) in media_deltas {
            if let Some(component) = self.state.media.composition.dissolved_components.iter_mut().find(|c| c.molecule_id == molecule_id) {
                component.concentration.value = (component.concentration.value + delta).max(0.0);
            }
        }

        Ok(())
    }

    fn execute_command(&mut self, command: Command) -> Result<(), BioforgeError> {
        match command {
            Command::AdvanceToNextStep => {
                self.current_step_index += 1;
                self.state.ticks_in_current_stage = 0;
                if let Some(next_method_id) =
                    self.process.default_workflow.get(self.current_step_index)
                {
                    println!("--- Entering stage: {} ---", next_method_id);
                } else {
                    println!("--- Reached end of process workflow ---");
                }
            }
            Command::SetTemperature { asset_id, celsius } => {
                if let Some(asset) = self.state.assets.get_mut(&asset_id) {
                    asset.temperature = celsius;
                }
            }
            Command::AdjustPh { asset_id, target_ph } => {
                if let Some(asset) = self.state.assets.get_mut(&asset_id) {
                    asset.ph = target_ph;
                }
            }
            Command::AddMaterial { asset_id: _, material_id, amount_grams } => {
                if let Some(component) = self.state.media.composition.dissolved_components.iter_mut().find(|c| c.molecule_id == material_id) {
                    let concentration_increase = amount_grams / self.state.media.volume.value;
                    component.concentration.value += concentration_increase;
                    self.state.events.push(SimulationEvent::MaterialAdded {
                        id: material_id.clone(),
                        amount: amount_grams,
                    });
                }
            }
            Command::SetOrganismGrowthMultiplier { organism_id, multiplier } => {
                self.growth_multipliers.insert(organism_id, multiplier);
            }
        }
        Ok(())
    }

    fn evaluate_condition(&self, condition: &Condition) -> Result<bool, BioforgeError> {
        Ok(match condition {
            Condition::TimeInStage { ticks } => self.state.ticks_in_current_stage >= *ticks,
            Condition::BiomassStationary { threshold, window } => {
                if self.biomass_history.len() < *window {
                    return Ok(false);
                }
                let latest_biomass = self.biomass_history.back().unwrap();
                let past_biomass = self.biomass_history.iter().rev().nth(*window - 1).unwrap();
                
                if *past_biomass == 0.0 {
                    return Ok(false);
                }

                let growth_rate = (latest_biomass - past_biomass) / past_biomass / (*window as f64);
                growth_rate < *threshold
            }
            Condition::ProductAmount {
                molecule_name,
                target_grams,
            } => {
                let mut produced_grams = 0.0;
                for (org_id, org_state) in &self.state.organisms.states {
                    if let Some(org_def) = self.organism_defs.get(org_id) {
                        if let Some(yield_mg_g) = find_yield(org_def, molecule_name) {
                            produced_grams += org_state.biomass.value * yield_mg_g / 1000.0;
                        }
                    }
                }
                produced_grams >= *target_grams
            }
            Condition::MediaValue {
                molecule_id,
                operator,
                value,
            } => {
                if let Some(component) = self.state.media.composition.dissolved_components.iter().find(|c| c.molecule_id == *molecule_id) {
                    let current_value = component.concentration.value;
                    match operator {
                        ComparisonOperator::LessThan => current_value < *value,
                        ComparisonOperator::GreaterThan => current_value > *value,
                        ComparisonOperator::EqualTo => (current_value - value).abs() < f64::EPSILON,
                        ComparisonOperator::NotEqualTo => (current_value - value).abs() >= f64::EPSILON,
                    }
                } else {
                    false
                }
            }
            Condition::AssetValue {
                asset_id,
                parameter,
                operator,
                value,
            } => {
                if let Some(asset) = self.state.assets.get(asset_id) {
                    let current_value = match parameter.as_str() {
                        "temperature" => asset.temperature,
                        "ph" => asset.ph,
                        _ => return Ok(false),
                    };
                    match operator {
                        ComparisonOperator::LessThan => current_value < *value,
                        ComparisonOperator::GreaterThan => current_value > *value,
                        ComparisonOperator::EqualTo => (current_value - value).abs() < f64::EPSILON,
                        ComparisonOperator::NotEqualTo => {
                            (current_value - value).abs() >= f64::EPSILON
                        }
                    }
                } else {
                    false
                }
            }
        })
    }

    pub fn get_tick(&self) -> u64 {
        self.state.tick
    }

    pub fn get_assets(&self) -> &HashMap<String, LiveAsset> {
        &self.state.assets
    }

    pub fn get_assets_mut(&mut self) -> &mut HashMap<String, LiveAsset> {
        &mut self.state.assets
    }

    pub fn get_organism_states(&self) -> &HashMap<String, IndividualOrganismState> {
        &self.state.organisms.states
    }

    pub fn get_media_state(&self) -> &MediaState {
        &self.state.media
    }

    pub fn get_process(&self) -> &Process {
        &self.process
    }
}

fn find_yield(organism: &Organism, molecule_name: &str) -> Option<f64> {
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