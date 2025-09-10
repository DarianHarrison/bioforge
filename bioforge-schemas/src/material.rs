use crate::tea_lca;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialClass {
    Chemical,
    Biological,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MaterialCategory {
    PurchasedRawMaterial,
    ProcessIntermediate,
    FinalProduct,
    ProcessGeneratedByproduct,
    InternalSimulationState,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Identifiers {
    pub cas_number: Option<String>,
    pub chebi_id: Option<String>,
    pub pubchem_cid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub process_role: String,
    pub vendor: Option<String>,
    pub part_number: Option<String>,
    pub notes: Option<String>,
    pub identifiers: Option<Identifiers>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Specification {
    pub key: String,
    pub value: f64,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormulationType {
    Solution,
    Mixture,
    Hydrate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulationComponent {
    pub component_id: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Formulation {
    pub formulation_type: FormulationType,
    pub solvent_id: Option<String>,
    pub components: Vec<FormulationComponent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Material {
    pub material_id: String,
    pub material_name: String,
    pub material_class: MaterialClass,
    pub material_subtype: String,
    pub material_category: MaterialCategory,
    pub unit: String,
    pub metadata: Metadata,
    pub specifications: Vec<Specification>,
    pub formulation: Option<Formulation>,
    pub techno_economic_and_lca_profile: tea_lca::TechnoEconomicAndLcaProfile,
}