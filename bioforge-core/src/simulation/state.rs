use bioforge_schemas::{
    asset::Asset,
    environment::MediaState,
    organism_state::OrganismState,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SimulationEvent {
    MaterialConsumed { id: String, amount: f64 },
    MaterialAdded { id: String, amount: f64 },
}

#[derive(Debug, Clone)]
pub struct LiveAsset {
    pub definition: Asset,
    pub temperature: f64,
    pub ph: f64,
}

#[derive(Debug, Clone)]
pub struct SimulationState {
    pub tick: u64,
    pub ticks_in_current_stage: u64,
    pub assets: HashMap<String, LiveAsset>,
    pub media: MediaState,
    pub organisms: OrganismState,
    pub events: Vec<SimulationEvent>,
}