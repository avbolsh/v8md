use crate::model::{CacheIndex, MdObject};
use std::fs;
use std::path::PathBuf;

fn projects_root() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".v8md")
        .join("projects")
}

/// Project name must be safe to use as a single path component.
pub fn valid_project_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.contains('*')
        && !name.contains('?')
        && !name.contains('"')
        && !name.contains('<')
        && !name.contains('>')
        && !name.contains('|')
        && name != "."
        && name != ".."
}

fn project_dir(name: &str) -> Option<PathBuf> {
    if valid_project_name(name) {
        Some(projects_root().join(name))
    } else {
        None
    }
}

pub struct ProjectInfo {
    pub name: String,
    pub config_name: String,
    pub config_version: String,
}

pub fn list_projects() -> Vec<ProjectInfo> {
    let root = projects_root();
    let mut result: Vec<ProjectInfo> = Vec::new();
    if let Ok(entries) = fs::read_dir(&root) {
        for entry in entries.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let name = match entry.file_name().into_string() {
                Ok(n) => n,
                Err(_) => continue,
            };
            let (config_name, config_version) = match fs::read_to_string(entry.path().join("index.json")) {
                Ok(data) => serde_json::from_str::<CacheIndex>(&data)
                    .map(|idx| (idx.config_name, idx.config_version))
                    .unwrap_or_default(),
                Err(_) => (String::new(), String::new()),
            };
            result.push(ProjectInfo {
                name,
                config_name,
                config_version,
            });
        }
    }
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

pub fn index_path(name: &str) -> Option<PathBuf> {
    project_dir(name).map(|d| d.join("index.json"))
}

pub fn load_cache_index(project: &str) -> Option<CacheIndex> {
    let path = index_path(project)?;
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_cache_index(project: &str, index: &CacheIndex) -> std::io::Result<()> {
    let dir = match project_dir(project) {
        Some(d) => d,
        None => return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid project name: {}", project),
        )),
    };
    fs::create_dir_all(&dir)?;
    let path = dir.join("index.json");
    let data = serde_json::to_string_pretty(index)?;
    fs::write(path, data)
}

fn sub_dir_for_kind(kind: &str) -> &str {
    match kind {
        "Document" => "documents",
        "Catalog" => "catalogs",
        "InformationRegister" => "information_registers",
        "AccumulationRegister" => "accumulation_registers",
        "ChartOfCharacteristicTypes" => "charts_of_characteristic_types",
        "Enum" => "enums",
        "Constant" => "constants",
        "DocumentJournal" => "document_journals",
        "ExchangePlan" => "exchange_plans",
        other => other,
    }
}

pub fn save_object(project: &str, obj: &MdObject) -> std::io::Result<String> {
    let dir = match project_dir(project) {
        Some(d) => d,
        None => return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid project name: {}", project),
        )),
    };
    let dir = dir.join(sub_dir_for_kind(&obj.kind));
    fs::create_dir_all(&dir)?;

    let file_name = format!("{}.json", obj.name);
    let path = dir.join(&file_name);
    let data = serde_json::to_string_pretty(obj)?;
    fs::write(&path, data)?;

    Ok(format!("{}/{}", obj.kind, file_name))
}

pub fn load_object(project: &str, kind: &str, name: &str) -> Option<MdObject> {
    let dir = project_dir(project)?;
    let path = dir.join(sub_dir_for_kind(kind)).join(format!("{}.json", name));
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Removes all data of the given project (called before a fresh load).
pub fn clear_cache(project: &str) -> std::io::Result<()> {
    if let Some(dir) = project_dir(project) {
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
    }
    Ok(())
}

pub fn find_object(project: &str, query: &str) -> Option<MdObject> {
    if !valid_project_name(project) {
        return None;
    }
    let index = load_cache_index(project)?;

    let (kind, name) = if query.contains('.') {
        let parts: Vec<&str> = query.splitn(2, '.').collect();
        (parts[0], parts[1])
    } else {
        let entry = index.objects.iter().find(|e| e.name == query)?;
        (entry.kind.as_str(), entry.name.as_str())
    };

    load_object(project, kind, name)
}
