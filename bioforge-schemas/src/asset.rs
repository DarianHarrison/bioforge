use crate::{environment::Measurement, tea_lca};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowCapacity {
    pub direction: i32,
    pub rate: Measurement<f64>,
    pub material_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionPoint {
    pub port_id: String,
    pub port_type: Option<String>,
    pub description: Option<String>,
    pub flow_capacities: Vec<FlowCapacity>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlParameter {
    pub key: String,
    pub value: f64,
    pub unit: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub default: Option<f64>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitoredVariable {
    pub key: String,
    pub value: f64,
    pub unit: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub default: Option<f64>,
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalTask {
    pub task_id: String,
    pub task_name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReliabilityModel {
    pub mtbf: Measurement<f64>,
    pub mttr: Measurement<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    TimeBased,
    UsageBased,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceTrigger {
    pub trigger_type: TriggerType,
    pub unit: String,
    pub interval: i64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreventativeMaintenanceTask {
    pub task_id: String,
    pub task_name: String,
    pub trigger: MaintenanceTrigger,
    pub materials_and_parts: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaintenanceProfile {
    pub reliability_model: Option<ReliabilityModel>,
    pub preventative_schedules: Option<Vec<PreventativeMaintenanceTask>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaborRequirement {
    pub linked_task_id: String,
    pub task_description: String,
    pub required_role_id: String,
    pub duration: Measurement<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerModel {
    pub description: Option<String>,
    pub operating_power: Measurement<f64>,
    pub standby_power: Measurement<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationalParameters {
    pub configuration_and_control: Option<Vec<ControlParameter>>,
    pub monitoring: Option<Vec<MonitoredVariable>>,
    pub operational_tasks: Option<Vec<OperationalTask>>,
    pub maintenance: Option<MaintenanceProfile>,
    pub labor_requirements: Option<Vec<LaborRequirement>>,
    pub power_model: Option<PowerModel>,
}

/// Represents the digital twin of any physical piece of equipment across the entire bioprocess
/// value chain.
///
/// This includes **upstream hardware like fermenters, downstream units like
/// chromatography skids, and finishing equipment for formulation, filling, packaging, storage,
/// and quality control.**
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    /// A unique, machine-readable identifier for the asset (e.g., "SFE-SYSTEM-01").
    pub asset_id: String,
    /// A human-readable name for display purposes (e.g., "Supercritical Fluid Extraction System").
    pub display_name: Option<String>,
    /// A standardized category for the asset's function (e.g., "BIOREACTOR", "CHROMATOGRAPHY_SKID").
    pub asset_type: String,
    /// A high-level process area where the asset is used (e.g., "CULTIVATION", "DOWNSTREAM").
    pub group: Option<String>,
    /// A brief description of the asset's purpose and capabilities.
    pub description: Option<String>,
    /// Defines the physical or logical connection points for integrating with other assets.
    pub connection_points: Option<Vec<ConnectionPoint>>,
    /// Contains all parameters related to the asset's operation, control, and maintenance.
    pub operational_parameters: Option<OperationalParameters>,
    /// The unified Techno-Economic Analysis (TEA) and Life Cycle Assessment (LCA) profile for the asset.
    pub techno_economic_and_lca_profile: Option<tea_lca::TechnoEconomicAndLcaProfile>,
}