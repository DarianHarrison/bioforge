# bioforge
A Framework for Sustainable Bioprocess Design

Welcome to BioForge! This document explains the end-to-end workflow of the simulation application, providing a clear overview of what happens from the initial user request to the final analysis and report generation.

## How to RUn
```bash
cargo run -p bioforge-app
```

## High-Level Overview

At its core, BioForge is a digital twin for bioprocess engineering. It takes a high-level goal—such as producing specific molecules using defined process grades—and intelligently designs, simulates, and analyzes the entire upstream and downstream process required to achieve that goal.

The workflow can be visualized as follows:

`User Request (request.yaml)` -> `JIT Optimization` -> `Upstream Simulation` -> `Downstream Simulation` -> `Analysis & Reports`

Note: Currently only supports the following targets
```
[
    ('LUT-NUT-01', 'Lutein - Nutraceutical Grade'),
    ('LUT-FOD-01', 'Lutein - Food Colorant Grade'),
    ('LUT-FED-01', 'Lutein - Animal Feed Grade'),
    ('LUT-NUT-01-GOLD', 'Lutein - Nutraceutical Grade (Gold Standard)'),
    ('LUT-NUT-01-ECON', 'Lutein - Nutraceutical Grade (Econo-Focused)'),
    ('LUT-NUT-01-ECO', 'Lutein - Nutraceutical Grade (Eco-Focused)'),
    ('LUT-FOD-01-ECO', 'Lutein - Food Colorant Grade (Eco-Focused)'),
    ('LUT-FED-01-ECON', 'Lutein - Animal Feed Grade (Econo-Focused)'),
    ('BGL-NUT-01', 'β-Glucan - Nutraceutical Grade'),
    ('BGL-COS-01', 'β-Glucan - Cosmetic Grade'),
    ('BGL-NUT-01-ECO', 'β-Glucan - Nutraceutical Grade (Eco-Focused)'),
    ('BGL-COS-01-ECO', 'β-Glucan - Cosmetic Grade (Eco-Focused)')
]
```

---

## Step-by-Step Breakdown

Here is a more detailed look at each stage of the automated workflow.

### 1. The User Request

The process begins with a **`ValorizationRequest`** defined by the user in the `bioforge-app/request.yaml` file. This request is highly configurable and specifies:
* **Target Molecules**: The desired products (e.g., Lutein, beta-glucans).
* **Process Grade**: The exact downstream process to be used for each target, selected from a comprehensive list of production and alternative workflows (e.g., `PROC-LUT-NUTRA-01` for nutraceutical-grade Lutein).
* **Objective**: The primary goal for the process, such as maximizing yield.

### 2. Just-In-Time (JIT) Optimization

The JIT module acts as the "brain" of the operation. It takes the user's request and consults the **Knowledge Base** (all the `.yaml` files) to make intelligent decisions.

* **Organism Selection**: It first selects the optimal combination of organisms—a **consortium**—best suited to produce the target molecules.
* **Downstream Process Selection**: Based on the explicit `process_id` provided in the user request, the JIT module selects the exact downstream workflows from the knowledge base.
* **Dynamic Media Formulation**: Based on the metabolic needs of the selected organisms, it dynamically generates a custom initial media formula, ensuring all necessary nutrients are available for growth. This formulation is saved for the upcoming simulation run and also for the bill of materials.

### 3. Upstream Simulation (Consortium Growth)

This stage simulates the cultivation of the selected organism consortium.

* **Unified Simulation**: A single simulation is run where all selected organisms grow together, sharing and interacting with the same media.
* **Dynamic Modeling**: The simulation engine models key biological processes on an hourly basis ("tick"), including nutrient consumption, biomass growth, and the secretion of metabolic byproducts into the media.
* **Data Logging & Visualization**: Time-series data is logged to a CSV file, and upon completion, a set of graphs is automatically generated to visualize the results, including biomass growth and media composition changes over time.

### 4. Downstream Simulation (Purification)

This stage simulates the extraction, purification, and formulation of the final products using the specific processes selected by the user.

* **Process Execution**: The simulation runs through the selected downstream workflows, modeling each unit operation (e.g., extraction, saponification, filtration).
* **Resource Tracking**: During this phase, the simulation logs the consumption of materials (e.g., solvents, buffers), energy, and labor required for each step.

### 5. Final Analysis & Reporting

In the final stage, the application aggregates all the data from both the upstream and downstream simulations to produce a comprehensive report.

* **Data Aggregation**: The system combines all resource usage into a final **Bill of Materials (BOM)**.
* **Techno-Economic & Life Cycle Analysis**: Using the aggregated data, the application calculates the final **Cost of Goods Sold (COGS)** and a **Life Cycle Assessment (LCA)**, which includes metrics like the process's carbon footprint.
* **Summary Report**: All of this information is presented to the user in a clear, formatted summary in the console, providing a complete overview of the simulated process from start to finish.
* **Process Visualization**: A flowchart of the selected downstream processes is generated and saved as an image file (`4_process_flow.png`) for easy review.