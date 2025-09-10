use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    SetTemperature {
        asset_id: String,
        celsius: f64,
    },
    AdjustPh {
        asset_id: String,
        target_ph: f64,
    },
    AdvanceToNextStep,
    AddMaterial {
        asset_id: String,
        material_id: String,
        amount_grams: f64,
    },
    SetOrganismGrowthMultiplier {
        organism_id: String,
        multiplier: f64,
    },
}