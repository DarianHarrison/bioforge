use crate::{
    asset::Asset, labor::LaborRole, material::Material, organism::Organism, process::Process,
    rule::Rule,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AssetFile {
    pub schema_version: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
pub struct MaterialFile {
    pub schema_version: String,
    pub materials: Vec<Material>,
}

#[derive(Debug, Deserialize)]
pub struct OrganismFile {
    pub schema_version: String,
    pub organisms: Vec<Organism>,
}

#[derive(Debug, Deserialize)]
pub struct LaborRoleFile {
    pub schema_version: String,
    pub labor_roles: Vec<LaborRole>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessFile {
    pub schema_version: String,
    pub processes: Vec<Process>,
}

#[derive(Debug, Deserialize)]
pub struct RuleFile {
    pub schema_version: String,
    pub rules: Vec<Rule>,
}