use crate::{
    error::BioforgeError,
    logger::TimeSeriesLogger,
    simulation::{
        engine::SimulationEngine,
        state::{LiveAsset, SimulationState},
    },
};
use bioforge_schemas::{
    asset::Asset,
    environment::{MediaState, Measurement},
    organism::Organism,
    organism_state::{IndividualOrganismState, OrganismState},
    process::Process,
    rule::Rule,
};
use std::collections::{HashMap, VecDeque};

/// A fluent builder for constructing a `SimulationEngine`.
///
/// This builder provides a step-by-step API to configure all necessary components
/// for a simulation, including assets, rules, processes, organisms, and initial media.
#[derive(Default)]
pub struct SimulationBuilder {
    assets: Vec<Asset>,
    rules: Vec<Rule>,
    process: Option<Process>,
    organisms: Vec<Organism>,
    initial_media: Option<MediaState>,
    log_path: Option<String>,
}

impl SimulationBuilder {
    /// Creates a new, empty `SimulationBuilder`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the collection of `Asset` definitions to be used in the simulation.
    pub fn with_assets(mut self, assets: Vec<Asset>) -> Self {
        self.assets = assets;
        self
    }

    /// Sets the collection of `Rule` definitions that govern the simulation's logic.
    pub fn with_rules(mut self, rules: Vec<Rule>) -> Self {
        self.rules = rules;
        self
    }

    /// Sets the `Process` definition that defines the workflow of the simulation.
    pub fn with_process(mut self, process: Process) -> Self {
        self.process = Some(process);
        self
    }

    /// Sets the collection of `Organism` definitions to participate in the simulation.
    pub fn with_organisms(mut self, organisms: Vec<Organism>) -> Self {
        self.organisms = organisms;
        self
    }

    /// Sets the initial `MediaState` for the simulation environment.
    pub fn with_initial_media(mut self, media: MediaState) -> Self {
        self.initial_media = Some(media);
        self
    }

    /// Configures the simulation to write time-series data to the specified CSV file.
    pub fn with_timeseries_logging_to_file(mut self, path: &str) -> Self {
        self.log_path = Some(path.to_string());
        self
    }

    /// Consumes the builder and returns a fully configured `SimulationEngine`.
    ///
    /// # Errors
    ///
    /// Returns a `BioforgeError` if essential components like organisms, media, or a process
    /// have not been provided.
    pub fn build(self) -> Result<SimulationEngine, BioforgeError> {
        if self.organisms.is_empty() {
            return Err(BioforgeError::NoOrganismProvided);
        }

        let mut initial_assets = HashMap::new();
        for asset_def in self.assets {
            initial_assets.insert(
                asset_def.asset_id.clone(),
                LiveAsset {
                    temperature: 25.0,
                    ph: 7.0,
                    definition: asset_def,
                },
            );
        }

        let rules_map = self.rules.into_iter().map(|r| (r.name.clone(), r)).collect();
        let organism_defs = self
            .organisms
            .iter()
            .map(|o| (o.organism_id.clone(), o.clone()))
            .collect::<HashMap<_, _>>();

        let initial_organism_states = self
            .organisms
            .iter()
            .map(|org| {
                let state = IndividualOrganismState {
                    biomass: Measurement {
                        value: org.initial_biomass.value,
                        unit: org.initial_biomass.unit.clone(),
                    },
                };
                (org.organism_id.clone(), state)
            })
            .collect();

        let organism_state = OrganismState {
            states: initial_organism_states,
        };

        let state = SimulationState {
            tick: 0,
            ticks_in_current_stage: 0,
            assets: initial_assets,
            media: self.initial_media.ok_or(BioforgeError::MediaNotDefined)?,
            organisms: organism_state,
            events: Vec::new(),
        };

        let logger = match self.log_path {
            Some(path) => Some(
                TimeSeriesLogger::new(&path)
                    .map_err(|e| BioforgeError::FileIO(path.clone(), e))?,
            ),
            None => None,
        };

        let growth_multipliers = organism_defs.keys().map(|id| (id.clone(), 1.0)).collect();

        Ok(SimulationEngine {
            state,
            process: self.process.ok_or(BioforgeError::ProcessNotDefined)?,
            rules: rules_map,
            organism_defs,
            current_step_index: 0,
            logger,
            biomass_history: VecDeque::new(),
            growth_multipliers,
        })
    }
}