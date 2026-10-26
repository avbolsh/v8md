use crate::model::{CacheIndex, MdObject};
use std::fs;
use std::path::PathBuf;

fn cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".v8md")
}

pub fn load_cache_index() -> Option<CacheIndex> {
    let path = cache_dir().join("index.json");
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_cache_index(index: &CacheIndex) -> std::io::Result<()> {
    let dir = cache_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join("index.json");
    let data = serde_json::to_string_pretty(index)?;
    fs::write(path, data)
}

pub fn save_object(obj: &MdObject) -> std::io::Result<String> {
    let sub_dir = match obj.kind.as_str() {
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
    };
    let dir = cache_dir().join(sub_dir);
    fs::create_dir_all(&dir)?;

    let file_name = format!("{}.json", obj.name);
    let path = dir.join(&file_name);
    let data = serde_json::to_string_pretty(obj)?;
    fs::write(&path, data)?;

    Ok(format!("{}/{}", obj.kind, file_name))
}

pub fn load_object(kind: &str, name: &str) -> Option<MdObject> {
    let sub_dir = match kind {
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
    };
    let path = cache_dir().join(sub_dir).join(format!("{}.json", name));
    if !path.exists() {
        return None;
    }
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn clear_cache() -> std::io::Result<()> {
    let dir = cache_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

pub fn find_object(query: &str) -> Option<MdObject> {
    let index = load_cache_index()?;

    let (kind, name) = if query.contains('.') {
        let parts: Vec<&str> = query.splitn(2, '.').collect();
        (parts[0], parts[1])
    } else {
        let entry = index.objects.iter().find(|e| e.name == query)?;
        (entry.kind.as_str(), entry.name.as_str())
    };

    load_object(kind, name)
}