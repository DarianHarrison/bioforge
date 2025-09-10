use crate::command::Command;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    LessThan,
    GreaterThan,
    EqualTo,
    NotEqualTo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    AssetValue {
        asset_id: String,
        parameter: String,
        operator: ComparisonOperator,
        value: f64,
    },
    TimeInStage {
        ticks: u64,
    },
    BiomassStationary {
        threshold: f64,
        window: usize,
    },
    ProductAmount {
        molecule_name: String,
        target_grams: f64,
    },
    MediaValue {
        molecule_id: String,
        operator: ComparisonOperator,
        value: f64,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub name: String,
    pub condition: Condition,
    pub action: Command,
}