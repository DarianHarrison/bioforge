//! Defines the data structures for representing an organism in the BioForge knowledge base.
//! This includes static properties like composition and morphology, as well as dynamic
//! parameters like environmental tolerances and metabolic rates.

use crate::environment::Measurement;
use serde::{Deserialize, Serialize};

/// Enumerates the high-level biological classifications for organisms in the simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrganismType {
    Bacteria,
    Microalgae,
    Microfungi,
    Phage,
    CellLine,
}

/// Contains details about a specific strain, including its origin and whether it has been genetically engineered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrainDetails {
    /// A brief description of the strain's lineage or key characteristics.
    pub description: Option<String>,
    /// A flag indicating if the organism has been genetically modified.
    pub is_engineered: bool,
}

/// Represents the elemental composition of the organism's biomass as a percentage of dry weight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementalComposition {
    pub carbon: f64,
    pub hydrogen: f64,
    pub oxygen: f64,
    pub nitrogen: f64,
    pub phosphorus: f64,
    pub sulfur: f64,
}

/// A summary of the major macromolecular components of the organism's biomass,
/// expressed as a percentage of dry weight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacromolecularSummary {
    pub protein: f64,
    pub carbohydrate: f64,
    pub lipid: f64,
    pub nucleic_acid: f64,
    pub ash: f64,
}

/// Describes the physical shape and size of the organism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Morphology {
    /// The typical diameter of a single cell or organism unit.
    pub nominal_diameter: Measurement<f64>,
}

/// Defines the yield of a specific target molecule produced by the organism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetMoleculeYield {
    /// The common name of the molecule (e.g., "Lutein").
    pub molecule: String,
    /// The concentration of the molecule in milligrams per gram of the organism's dry weight.
    pub concentration_mg_g_dw: f64,
}

/// A collection of target molecules, grouped by their chemical class.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TargetedMolecularClasses {
    pub terpenoids_and_carotenoids: Vec<TargetMoleculeYield>,
    pub cell_wall_components: Vec<TargetMoleculeYield>,
}

/// Encapsulates the static, inherent properties of an organism that do not change during simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaticProperties {
    pub elemental_composition: ElementalComposition,
    pub macromolecular_summary: MacromolecularSummary,
    pub morphology: Morphology,
    pub targeted_molecular_classes: TargetedMolecularClasses,
}

/// A generic struct to define a minimum and maximum tolerance range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToleranceRange<T> {
    pub min: T,
    pub max: T,
}

/// Defines the organism's response to light for photosynthesis.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotosyntheticLightResponse {
    /// The range of photosynthetically active radiation (PAR) wavelengths.
    pub par_wavelength_range_nm: (u32, u32),
    /// The light intensity at which the photosynthetic rate is maximized.
    pub saturation_ppfd: Measurement<f64>,
    /// The light intensity above which the photosynthetic rate begins to decline.
    pub photoinhibition_ppfd: Measurement<f64>,
}

/// Defines the organism's tolerance to temperature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemperatureTolerance {
    /// The optimal temperature for growth.
    pub optimal: Measurement<f64>,
    /// The viable temperature range for the organism.
    pub range: ToleranceRange<f64>,
}

/// Defines the organism's tolerance to pH.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PHTolerance {
    /// The optimal pH for growth.
    pub optimal: f64,
    /// The viable pH range for the organism.
    pub range: ToleranceRange<f64>,
}

/// Defines the organism's tolerance to a specific chemical compound.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChemicalTolerance {
    pub molecule_id: String,
    pub molecule_name: String,
    pub minimum_inhibitory_concentration: Option<Measurement<f64>>,
    pub inhibitory_concentration_50: Option<Measurement<f64>>,
}

/// A collection of all environmental tolerances for the organism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvironmentalTolerances {
    pub photosynthetic_light_response: Option<PhotosyntheticLightResponse>,
    pub temperature: TemperatureTolerance,
    pub ph: PHTolerance,
    pub chemical: Vec<ChemicalTolerance>,
}

/// Enumerates the aeration conditions for metabolic activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AerationCondition {
    Aerobic,
    Anaerobic,
    MicroAerobic,
    Anoxic,
}

/// Enumerates the light conditions for metabolic activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LightCondition {
    Light,
    Dark,
}

/// Defines the specific environmental conditions under which a metabolic exchange rate is valid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExchangeConditions {
    pub aeration: AerationCondition,
    pub light: Option<LightCondition>,
    pub notes: Option<String>,
}

/// Defines the rate of consumption or secretion of a dissolved component from the media.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaExchangeRate {
    pub molecule_id: String,
    pub molecule_name: String,
    pub max_exchange_rate: Measurement<f64>,
    pub conditions: ExchangeConditions,
}

/// Defines the rate of consumption or secretion of a gas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GasExchangeRate {
    pub gas_id: String,
    pub gas_name: String,
    pub max_exchange_rate: Measurement<f64>,
    pub conditions: ExchangeConditions,
}

/// Encapsulates all metabolic exchange rates for an organism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetabolicExchange {
    pub media_consumption: Vec<MediaExchangeRate>,
    pub media_secretion: Vec<MediaExchangeRate>,
    pub gas_consumption: Vec<GasExchangeRate>,
    pub gas_secretion: Vec<GasExchangeRate>,
}

/// Encapsulates the dynamic parameters of an organism that influence its behavior during simulation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicParameters {
    pub growth_rate_per_hr: f64,
    pub environmental_tolerances: EnvironmentalTolerances,
    pub metabolic_exchange: MetabolicExchange,
}

/// The top-level struct representing a complete organism definition in the knowledge base.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Organism {
    pub organism_id: String,
    pub organism_name: String,
    pub organism_type: OrganismType,
    pub strain_details: Option<StrainDetails>,
    pub initial_biomass: Measurement<f64>,
    pub static_properties: StaticProperties,
    pub dynamic_parameters: DynamicParameters,
}