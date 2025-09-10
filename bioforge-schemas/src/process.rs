use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RequiredMaterial {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QcCheck {
    pub method_id: String,
    pub timing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Method {
    pub method_id: String,
    pub stage: String,
    pub technique: String,
    pub required_asset_id: String,
    pub operating_parameters: HashMap<String, serde_json::Value>,
    pub required_materials: Vec<RequiredMaterial>,
    pub qc_checks: Vec<QcCheck>,
    pub required_rule_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Process {
    pub process_id: String,
    pub process_name: String,
    pub component_class: String,
    pub status: String,
    pub notes: String,
    pub default_workflow: Vec<String>,
    pub methods: Vec<Method>,
}