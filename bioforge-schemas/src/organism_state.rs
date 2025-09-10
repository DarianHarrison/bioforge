use crate::environment::Measurement;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualOrganismState {
    pub biomass: Measurement<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganismState {
    pub states: HashMap<String, IndividualOrganismState>,
}