use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement<T> {
    pub value: T,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GasComponent {
    pub gas_id: String,
    pub gas_name: String,
    pub concentration: Measurement<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Aeration {
    pub flow_rate: Measurement<f64>,
    pub gas_composition_percent: Option<Vec<GasComponent>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpectralIrradiancePoint {
    pub value: f64,
    pub nm: (u32, u32),
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhysicalConditions {
    pub surface_area: Option<Measurement<f64>>,
    pub volume: Option<Measurement<f64>>,
    pub spectral_irradiance: Option<Vec<SpectralIrradiancePoint>>,
    pub temperature: Measurement<f64>,
    pub aeration: Aeration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DissolvedComponent {
    pub molecule_id: String,
    pub molecule_name: String,
    pub concentration: Measurement<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DissolvedGas {
    pub gas_id: String,
    pub gas_name: String,
    pub concentration: Measurement<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaComposition {
    pub dissolved_components: Vec<DissolvedComponent>,
    pub dissolved_gases: Vec<DissolvedGas>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaState {
    pub volume: Measurement<f64>,
    pub ph: f64,
    pub composition: MediaComposition,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub environment_id: String,
    pub timestamp: i64,
    pub physical_conditions: PhysicalConditions,
    pub media_state: MediaState,
}