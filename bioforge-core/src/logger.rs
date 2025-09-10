use crate::simulation::state::SimulationState;
use csv::Writer;
use serde::Serialize;
use std::fs;
use std::io;

#[derive(Debug, Serialize)]
struct LogEntry {
    tick: u64,
    stage_id: String,
    organisms_json: String,
    media_volume_l: f64,
    media_ph: f64,
    dissolved_components_json: String,
    dissolved_gases_json: String,
    asset_states_json: String,
    events_json: String,
}

pub struct TimeSeriesLogger {
    writer: Writer<fs::File>,
}

impl TimeSeriesLogger {
    pub fn new(path: &str) -> Result<Self, io::Error> {
        let writer = Writer::from_path(path)?;
        Ok(Self { writer })
    }

    pub fn log_state(&mut self, state: &SimulationState, stage_id: &str) -> Result<(), anyhow::Error> {
        let asset_states_json = serde_json::to_string(
            &state
                .assets
                .iter()
                .map(|(id, asset)| {
                    (
                        id.clone(),
                        serde_json::json!({ "temperature": asset.temperature, "ph": asset.ph }),
                    )
                })
                .collect::<serde_json::Map<String, serde_json::Value>>(),
        )?;

        let organisms_json = serde_json::to_string(&state.organisms.states)?;
        let events_json = serde_json::to_string(&state.events)?;
        let dissolved_components_json = serde_json::to_string(&state.media.composition.dissolved_components)?;
        let dissolved_gases_json = serde_json::to_string(&state.media.composition.dissolved_gases)?;

        let entry = LogEntry {
            tick: state.tick,
            stage_id: stage_id.to_string(),
            organisms_json,
            media_volume_l: state.media.volume.value,
            media_ph: state.media.ph,
            dissolved_components_json,
            dissolved_gases_json,
            asset_states_json,
            events_json,
        };

        self.writer.serialize(entry)?;
        self.writer.flush()?;
        Ok(())
    }
}