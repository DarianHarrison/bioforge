//! This module is responsible for generating all visualizations from simulation log data.

use anyhow::Result;
use bioforge_core::analysis::{CogsResult, LcaResult, LogEntry};
use bioforge_core::simulation::state::SimulationEvent;
use bioforge_schemas::{
    environment::{DissolvedComponent, DissolvedGas},
    organism_state::IndividualOrganismState,
    process::Process,
    rule::{Condition, Rule},
};
use plotters::prelude::*;
use std::collections::HashMap;
use csv;
use serde_json;
use std::f64::consts::PI;


/// A flattened structure to hold all the parsed data from a single log record for easy plotting.
#[derive(Clone, Debug)]
struct PlottingData {
    tick: u64,
    biomass: HashMap<String, f64>,
    media_ph: f64,
    temperature: f64,
    dissolved_components: HashMap<String, f64>,
    dissolved_gases: HashMap<String, f64>,
    events: Vec<SimulationEvent>,
}

/// The main function to generate and save all plots for a simulation run.
pub fn generate_all_plots(
    output_dir: &str,
    log_path: &str,
    _cogs: &CogsResult,
    _lca: &LcaResult,
    organism_names: HashMap<String, String>,
) -> Result<()> {
    println!("[Plotting] Generating graphs from simulation data...");

    let data = parse_log_file(log_path)?;

    if data.is_empty() {
        println!("[Plotting] Warning: No data to plot.");
        return Ok(());
    }

    plot_biomass_growth(output_dir, &data, &organism_names)?;
    plot_media_composition(output_dir, &data)?;
    plot_environmental_parameters(output_dir, &data)?;
    plot_upstream_timeline(output_dir, &data)?;

    println!("[Plotting] Upstream graphs have been saved to '{}'.", output_dir);
    Ok(())
}

/// Parses the simulation log CSV file into a vector of `PlottingData` structs.
fn parse_log_file(log_path: &str) -> Result<Vec<PlottingData>> {
    let mut reader = csv::Reader::from_path(log_path)?;
    let mut data = Vec::new();

    for result in reader.deserialize() {
        let record: LogEntry = result?;
        let organisms: HashMap<String, IndividualOrganismState> =
            serde_json::from_str(&record.organisms_json)?;
        let dissolved_components: Vec<DissolvedComponent> =
            serde_json::from_str(&record.dissolved_components_json)?;
        let dissolved_gases: Vec<DissolvedGas> =
            serde_json::from_str(&record.dissolved_gases_json)?;
        let events: Vec<SimulationEvent> =
            serde_json::from_str(&record.events_json)?;

        let biomass = organisms
            .into_iter()
            .map(|(id, state)| (id, state.biomass.value))
            .collect();

        let dissolved_components_map = dissolved_components
            .into_iter()
            .map(|c| (c.molecule_name, c.concentration.value))
            .collect();

        let dissolved_gases_map = dissolved_gases
            .into_iter()
            .map(|g| (g.gas_name, g.concentration.value))
            .collect();

        let asset_states: HashMap<String, serde_json::Value> =
            serde_json::from_str(&record.asset_states_json)?;
        let temperature = asset_states
            .values()
            .next()
            .and_then(|v| v["temperature"].as_f64())
            .unwrap_or(25.0);

        data.push(PlottingData {
            tick: record.tick,
            biomass,
            media_ph: record.media_ph,
            temperature,
            dissolved_components: dissolved_components_map,
            dissolved_gases: dissolved_gases_map,
            events,
        });
    }

    Ok(data)
}

/// Generates a stacked area chart of biomass growth for each organism over time.
fn plot_biomass_growth(
    output_dir: &str,
    data: &[PlottingData],
    organism_names: &HashMap<String, String>,
) -> Result<()> {
    let path = format!("{}/1_biomass_growth.png", output_dir);
    let root = BitMapBackend::new(&path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_tick = data.last().map_or(1, |d| d.tick);
    let max_biomass: f64 = data
        .iter()
        .map(|d| d.biomass.values().sum::<f64>())
        .fold(0.0, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption("Biomass Growth Over Time", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(0u64..max_tick, 0f64..max_biomass * 1.1)?;

    chart.configure_mesh()
        .x_desc("Time (hours)")
        .y_desc("Biomass (g)")
        .draw()?;

    let colors = [RED, GREEN, BLUE, YELLOW, CYAN, MAGENTA];
    
    let mut sorted_organism_ids: Vec<_> = organism_names.keys().cloned().collect();
    sorted_organism_ids.sort();

    for (i, org_id) in sorted_organism_ids.iter().enumerate() {
        let org_name = organism_names.get(org_id).unwrap();
        let color = colors[i % colors.len()].clone();
        
        chart.draw_series(LineSeries::new(
            data.iter().map(|d| (d.tick, d.biomass.get(org_id).cloned().unwrap_or(0.0))),
            color.stroke_width(2),
        ))?
            .label(org_name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.filled()));
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

/// Generates a stacked area chart of key media components over time.
fn plot_media_composition(output_dir: &str, data: &[PlottingData]) -> Result<()> {
    let path = format!("{}/2_media_composition.png", output_dir);
    let root = BitMapBackend::new(&path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_tick = data.last().map_or(1, |d| d.tick);
    
    let components_to_plot = ["D-glucose", "sucrose", "acetate"];
    let max_concentration: f64 = data
        .iter()
        .map(|d| {
            components_to_plot.iter().map(|&name| d.dissolved_components.get(name).cloned().unwrap_or(0.0)).sum::<f64>()
        })
        .fold(0.0, f64::max);


    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Media Composition Over Time",
            ("sans-serif", 50).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(0u64..max_tick, 0f64..max_concentration * 1.1)?;

    chart
        .configure_mesh()
        .x_desc("Time (hours)")
        .y_desc("Concentration (g/L)")
        .draw()?;

    
    let colors = [BLUE, RED, GREEN, YELLOW];

    for (i, &component_name) in components_to_plot.iter().enumerate() {
        let color = colors[i % colors.len()].clone();
        
        chart.draw_series(LineSeries::new(
            data.iter().map(|d| (d.tick, d.dissolved_components.get(component_name).cloned().unwrap_or(0.0))),
            color.stroke_width(2),
        ))?
            .label(component_name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.filled()));
    }


    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}

/// Generates line charts for key environmental parameters over time.
fn plot_environmental_parameters(output_dir: &str, data: &[PlottingData]) -> Result<()> {
    let path = format!("{}/3_environmental_parameters.png", output_dir);
    let root = BitMapBackend::new(&path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_tick = data.last().map_or(1, |d| d.tick);

    let mut chart = ChartBuilder::on(&root)
        .caption(
            "Environmental Parameters Over Time",
            ("sans-serif", 50).into_font(),
        )
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(50)
        .build_cartesian_2d(0u64..max_tick, 0f64..100f64)?;

    chart
        .configure_mesh()
        .x_desc("Time (hours)")
        .y_desc("Value")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|d| (d.tick, d.media_ph)),
            RED.stroke_width(3),
        ))?
        .label("pH")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED.filled()));
    
    chart
        .draw_series(LineSeries::new(
            data.iter().map(|d| (d.tick, d.temperature)),
            BLUE.stroke_width(3),
        ))?
        .label("Temperature (°C)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE.filled()));

    let temp_series = (0..=max_tick).map(|x| {
        let angle = 2.0 * PI * (x as f64) / 24.0;
        (x, 25.0 + 5.0 * angle.sin()) // Sine wave for temperature
    });
    chart.draw_series(DashedLineSeries::new(temp_series, 5, 5, (&BLUE).into()))?
        .label("Idealized Temperature (°C)")
        .legend(|(x, y)| {
            PathElement::new(vec![(x, y), (x + 20, y)], BLUE.filled())
        });


    let light_series = (0..=max_tick).map(|x| {
        let angle = 2.0 * PI * (x as f64) / 24.0;
        (x, 50.0 + 50.0 * angle.sin())
    });
    chart.draw_series(DashedLineSeries::new(light_series, 5, 5, (&BLACK).into()))?
        .label("Photosynthetically Active Radiation (PAR)")
        .legend(|(x,y)| {
            PathElement::new(vec![(x,y), (x+20,y)], BLACK.filled())
        });


    chart
        .draw_series(LineSeries::new(
            data.iter()
                .map(|d| (d.tick, d.dissolved_gases.get("oxygen").cloned().unwrap_or(0.0) * 1000.0)),
            GREEN.stroke_width(3),
        ))?
        .label("Dissolved O2 (mg/L)")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN.filled()));

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;
    root.present()?;
    Ok(())
}


/// Generates a flowchart of the end-to-end process.
pub fn plot_process_flow(output_dir: &str, processes: &[&Process], rules: &HashMap<String, Rule>) -> Result<()> {
    let path = format!("{}/4_process_flow.png", output_dir);
    let root_area = BitMapBackend::new(&path, (1920, 1080)).into_drawing_area();
    root_area.fill(&WHITE)?;
    let title = format!("Process Flow: {}", processes.iter().map(|p| p.process_name.as_str()).collect::<Vec<&str>>().join(" & "));
    root_area.titled(&title, ("sans-serif", 40))?;

    let drawing_area = root_area.margin(20, 20, 60, 20);

    let mut current_y_offset = 100;

    for process in processes {
        let process_title_style = TextStyle::from(("sans-serif", 24).into_font()).color(&BLACK);
        drawing_area.draw_text(
            &format!("Process: {}", process.process_name),
            &process_title_style,
            (50, current_y_offset as i32),
        )?;
        current_y_offset += 40;

        let num_steps = process.default_workflow.len();
        if num_steps == 0 {
            current_y_offset += 100;
            continue;
        }

        let node_min_width = 180;
        let node_min_height = 80;
        let x_gap = 70;
        let x_padding = 10;

        let mut max_text_width_in_node = 0;
        let text_style = TextStyle::from(("sans-serif", 14).into_font());
        for method_id in process.default_workflow.iter() {
            let method = process.methods.iter().find(|m| &m.method_id == method_id).unwrap();
            let text_lines = vec![
                format!("Stage: {}", method.stage),
                format!("Method: {}", method.method_id),
                format!("Technique: {}", method.technique),
            ];
            for line in text_lines {
                if let Ok((w, _)) = drawing_area.estimate_text_size(&line, &text_style) {
                    if w > max_text_width_in_node {
                        max_text_width_in_node = w;
                    }
                }
            }
        }
        let node_width = (max_text_width_in_node as u32 + x_padding * 2).max(node_min_width as u32) as i32;
        let node_height = node_min_height;

        let total_flow_width = (num_steps as i32 * node_width) + ((num_steps as i32 - 1) * x_gap);
        let mut x_pos = (1920 / 2 - total_flow_width / 2) as i32;

        let mut last_node_end_x = 0;

        for (i, method_id) in process.default_workflow.iter().enumerate() {
            let method = process.methods.iter().find(|m| &m.method_id == method_id).unwrap();
            
            let top_left = (x_pos, current_y_offset as i32 + 50);
            
            let node_color = RGBColor(70, 130, 180);
            let style = ShapeStyle { color: node_color.into(), filled: true, stroke_width: 2 };
            drawing_area.draw(&Rectangle::new(
                [top_left, (top_left.0 + node_width, top_left.1 + node_height)],
                style,
            ))?;
            
            let text_style = TextStyle::from(("sans-serif", 14).into_font()).color(&WHITE);

            let text_start_x = x_pos + 10;
            let text_start_y = current_y_offset as i32 + 50 + 15;

            drawing_area.draw_text(&format!("Stage: {}", method.stage), &text_style, (text_start_x, text_start_y))?;
            drawing_area.draw_text(&format!("Method: {}", method.method_id), &text_style, (text_start_x, text_start_y + 20))?;
            drawing_area.draw_text(&format!("Technique: {}", method.technique), &text_style, (text_start_x, text_start_y + 40))?;
            
            if i > 0 {
                let arrow_y = current_y_offset as i32 + 50 + node_height / 2;
                let start_point = (last_node_end_x as i32, arrow_y);
                let end_point = (x_pos, arrow_y);

                drawing_area.draw(&PathElement::new(vec![start_point, end_point], BLACK.stroke_width(2)))?;
                
                let arrowhead_size = 10;
                let arrowhead_points = vec![
                    end_point,
                    (end_point.0 - arrowhead_size, end_point.1 - arrowhead_size / 2),
                    (end_point.0 - arrowhead_size, end_point.1 + arrowhead_size / 2),
                ];
                drawing_area.draw(&Polygon::new(arrowhead_points, BLACK.filled()))?;

                let prev_method = process.methods.iter().find(|m| m.method_id == process.default_workflow[i-1]).unwrap();
                let rule_text_content = if let Some(rule_ids) = &prev_method.required_rule_ids {
                    if let Some(rule_id) = rule_ids.get(0) {
                        if let Some(rule) = rules.get(rule_id) {
                            if let Condition::TimeInStage { ticks } = rule.condition {
                                format!("{} hours", ticks)
                            } else {
                                rule_id.clone()
                            }
                        } else {
                            rule_id.clone()
                        }
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                };

                
                if !rule_text_content.is_empty() {
                    let rule_text_style = TextStyle::from(("sans-serif", 12).into_font());
                    let text_mid_x = start_point.0 + (end_point.0 - start_point.0) / 2;
                    let text_offset_y = -20;
                    drawing_area.draw_text(&rule_text_content, &rule_text_style, (text_mid_x, arrow_y + text_offset_y))?;
                }
            }

            last_node_end_x = x_pos + node_width;
            x_pos += node_width + x_gap;
        }
        current_y_offset += node_height as u32 + 100;
    }

    root_area.present()?;
    Ok(())
}

/// Generates a timeline graph of the upstream simulation, highlighting material infusion events.
fn plot_upstream_timeline(
    output_dir: &str,
    data: &[PlottingData],
) -> Result<()> {
    let path = format!("{}/5_upstream_timeline.png", output_dir);
    let root = BitMapBackend::new(&path, (1024, 256)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_tick = data.last().map_or(1, |d| d.tick);

    let mut chart = ChartBuilder::on(&root)
        .caption("Upstream Infusion Events", ("sans-serif", 30).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(20)
        .build_cartesian_2d(0u64..max_tick, 0..2i32)?;

    chart.configure_mesh()
        .x_desc("Time (hours)")
        .disable_y_axis()
        .draw()?;

    // Extract infusion events
    let infusion_events: Vec<u64> = data.iter()
        .filter_map(|d| {
            if d.events.iter().any(|e| matches!(e, SimulationEvent::MaterialAdded { .. })) {
                Some(d.tick)
            } else {
                None
            }
        })
        .collect();

    // Draw the histogram
    chart.draw_series(
        Histogram::vertical(&chart)
            .style(BLUE.filled())
            .data(infusion_events.iter().map(|tick| (*tick, 1))),
    )?;

    root.present()?;
    Ok(())
}