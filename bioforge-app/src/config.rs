use anyhow::{Context, Result};
use bioforge_schemas::{
    asset::Asset,
    file_formats::{
        AssetFile, LaborRoleFile, MaterialFile, OrganismFile, ProcessFile, RuleFile,
    },
    labor::LaborRole,
    material::Material,
    organism::Organism,
    process::Process,
    rule::Rule,
};
use std::{collections::HashMap, fs, path::Path};

/// A container for all the static data loaded from YAML files.
/// This represents the complete "knowledge base" for a simulation run.
pub struct KnowledgeBase {
    pub assets: HashMap<String, Asset>,
    pub materials: HashMap<String, Material>,
    pub organisms: HashMap<String, Organism>,
    pub labor_roles: HashMap<String, LaborRole>,
    pub processes: HashMap<String, Process>,
    pub rules: HashMap<String, Rule>,
}

impl KnowledgeBase {
    /// Loads all data from the specified base directory.
    pub fn load(base_path: &str) -> Result<Self> {
        println!("Loading knowledge base from '{}'...", base_path);

        let assets = load_yaml_files_into_map(
            Path::new(base_path).join("3_assets"),
            |file: AssetFile| file.assets,
            |item: &Asset| item.asset_id.clone(),
        )?;
        let materials = load_yaml_files_into_map(
            Path::new(base_path).join("1_materials"),
            |file: MaterialFile| file.materials,
            |item: &Material| item.material_id.clone(),
        )?;
        let organisms = load_yaml_files_into_map(
            Path::new(base_path).join("2_organisms"),
            |file: OrganismFile| file.organisms,
            |item: &Organism| item.organism_id.clone(),
        )?;
        let labor_roles = load_yaml_files_into_map(
            Path::new(base_path).join("4_labor"),
            |file: LaborRoleFile| file.labor_roles,
            |item: &LaborRole| item.labor_role_id.clone(),
        )?;
        let processes = load_yaml_files_into_map(
            Path::new(base_path).join("5_processes"),
            |file: ProcessFile| file.processes,
            |item: &Process| item.process_id.clone(),
        )?;
        let rules = load_yaml_files_into_map(
            Path::new(base_path).join("6_rules"),
            |file: RuleFile| file.rules,
            |item: &Rule| item.name.clone(),
        )?;

        println!("Knowledge base loaded successfully.");
        Ok(Self {
            assets,
            materials,
            organisms,
            labor_roles,
            processes,
            rules,
        })
    }
}

/// Generic helper to load all YAML files in a directory into a HashMap.
fn load_yaml_files_into_map<P, F, E, T, K>(
    dir_path: P,
    extract_vec: E,
    get_key: K,
) -> Result<HashMap<String, T>>
where
    P: AsRef<Path>,
    F: for<'de> serde::Deserialize<'de>, // The file wrapper struct (e.g., AssetFile)
    E: Fn(F) -> Vec<T>,                  // A closure to extract the Vec<T> from the wrapper
    K: Fn(&T) -> String,                 // A closure to get the key for the map from an item T
{
    let mut map = HashMap::new();
    for entry in fs::read_dir(dir_path.as_ref())
        .with_context(|| format!("Failed to read directory: {:?}", dir_path.as_ref()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |s| s == "yaml" || s == "yml") {
            let content = fs::read_to_string(&path)?;
            let file_wrapper: F = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML from {:?}", path))?;
            
            for item in extract_vec(file_wrapper) {
                map.insert(get_key(&item), item);
            }
        }
    }
    Ok(map)
}