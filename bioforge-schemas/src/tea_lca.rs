use crate::environment::Measurement;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CostEntry {
    pub cost_type: String,
    pub value_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImpactEntry {
    pub metric: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManufacturingAndAcquisition {
    pub costs: Vec<CostEntry>,
    pub impacts: Vec<ImpactEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UseAndOperation {
    pub costs: Vec<CostEntry>,
    pub impacts: Vec<ImpactEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Maintenance {
    pub costs: Vec<CostEntry>,
    pub impacts: Vec<ImpactEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndOfLife {
    pub costs: Vec<CostEntry>,
    pub impacts: Vec<ImpactEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifecycleStages {
    pub manufacturing_and_acquisition: ManufacturingAndAcquisition,
    pub use_and_operation: UseAndOperation,
    pub maintenance: Maintenance,
    pub end_of_life: EndOfLife,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnoEconomicAndLcaProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_lifespan: Option<Measurement<i32>>,
    pub lifecycle_stages: LifecycleStages,
}