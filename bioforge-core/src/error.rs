use thiserror::Error;

#[derive(Debug, Error)]
pub enum BioforgeError {
    #[error("Asset '{0}' not found in simulation state")]
    AssetNotFound(String),

    #[error("Organism definition for '{0}' not found")]
    OrganismNotFound(String),

    #[error("Process definition is missing")]
    ProcessNotDefined,

    #[error("Initial media state is missing")]
    MediaNotDefined,

    #[error("At least one organism must be provided for the simulation")]
    NoOrganismProvided,

    #[error("Could not find method '{0}' in process definition")]
    MethodNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigError(String), // Added the missing variant

    #[error("I/O error for file '{0}': {1}")]
    FileIO(String, #[source] std::io::Error),

    #[error("Failed to parse YAML from '{0}': {1}")]
    YamlParsing(String, #[source] serde_yaml::Error),

    // Correctly handle different error types from external crates
    #[error("Failed to parse JSON: {0}")]
    JsonParsing(#[from] serde_json::Error),

    #[error("Failed to process CSV file '{0}': {1}")]
    CsvError(String, #[source] csv::Error), // Correctly structured for context

    #[error("An error occurred during logging: {0}")]
    LoggingError(#[from] anyhow::Error), // Handles errors from the logger
}