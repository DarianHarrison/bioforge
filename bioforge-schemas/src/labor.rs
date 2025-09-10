use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnoEconomicProfile {
    pub cost_per_hour_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LaborRole {
    pub labor_role_id: String,
    pub role_name: String,
    pub skill_level: Option<i32>,
    pub description: Option<String>,
    pub techno_economic_profile: TechnoEconomicProfile,
}