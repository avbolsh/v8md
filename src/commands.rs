use crate::cache;
use crate::model::{MdObject, MdType};
use std::fs;
use std::path::Path;

pub fn load_dump(path: &Path, project: &str) -> Result<String, String> {
    if !cache::valid_project_name(project) {
        return Err(format!(
            "Invalid project name: '{}'. Name must be non-empty and contain no path characters (\\ / : * ? \" < > |) and not be '.' or '..'",
            project
        ));
    }
    let config_path = path.join("Configuration.xml");
    if !config_path.exists() {
        return Err(format!("Configuration.xml not found in {}", path.display()));
    }

    cache::clear_cache(project).map_err(|e| e.to_string())?;

    let mut objects: Vec<MdObject> = Vec::new();

    load_documents(path, &mut objects)?;
    load_catalogs(path, &mut objects)?;
    load_information_registers(path, &mut objects)?;
    load_accumulation_registers(path, &mut objects)?;
    load_charts_of_characteristic_types(path, &mut objects)?;
    load_enums(path, &mut objects)?;
    load_constants(path, &mut objects)?;
    load_document_journals(path, &mut objects)?;
    load_exchange_plans(path, &mut objects)?;

    let mut entries: Vec<crate::model::CacheIndexEntry> = Vec::new();
    for obj in &objects {
        let file = cache::save_object(project, obj).map_err(|e| e.to_string())?;
        entries.push(crate::model::CacheIndexEntry {
            kind: obj.kind.clone(),
            name: obj.name.clone(),
            file,
        });
    }

    let config_xml = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let (config_name, config_version) = parse_config_info(&config_xml);

    let index = crate::model::CacheIndex {
        config_name,
        config_version,
        objects: entries,
    };

    cache::save_cache_index(project, &index).map_err(|e| e.to_string())?;

    Ok(format!("Loaded {} objects into cache (project '{}')", index.objects.len(), project))
}

pub fn projects_list() -> Result<String, String> {
    let projects = cache::list_projects();
    if projects.is_empty() {
        return Ok("No projects loaded. Use 'v8md load-dump <path> --project <name>' first.".to_string());
    }

    let mut lines: Vec<String> = Vec::new();
    for p in &projects {
        if p.config_name.is_empty() {
            lines.push(p.name.clone());
        } else {
            lines.push(format!("{} — {} (version {})", p.name, p.config_name, p.config_version));
        }
    }

    Ok(lines.join("\n"))
}

fn parse_config_info(xml: &str) -> (String, String) {
    let mut name = String::new();
    let mut version = String::new();
    for line in xml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("<Name>") && trimmed.ends_with("</Name>") {
            name = trimmed
                .strip_prefix("<Name>")
                .unwrap()
                .strip_suffix("</Name>")
                .unwrap()
                .to_string();
        }
        if trimmed.starts_with("<Version>") && trimmed.ends_with("</Version>") {
            version = trimmed
                .strip_prefix("<Version>")
                .unwrap()
                .strip_suffix("</Version>")
                .unwrap()
                .to_string();
        }
        if !name.is_empty() && !version.is_empty() {
            break;
        }
    }
    (name, version)
}

fn load_documents(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let docs_dir = path.join("Documents");
    if !docs_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&docs_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_document(&data) {
                objects.push(obj);
            }
        }
    }

    Ok(())
}

fn load_catalogs(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let cats_dir = path.join("Catalogs");
    if !cats_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&cats_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_catalog(&data) {
                objects.push(obj);
            }
        }
    }

    Ok(())
}

fn load_information_registers(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let regs_dir = path.join("InformationRegisters");
    if !regs_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(&regs_dir).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_information_register(&data) {
                objects.push(obj);
            }
        }
    }

    Ok(())
}

fn load_accumulation_registers(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let dir = path.join("AccumulationRegisters");
    if !dir.exists() { return Ok(()); }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_accumulation_register(&data) {
                objects.push(obj);
            }
        }
    }
    Ok(())
}

fn load_charts_of_characteristic_types(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let dir = path.join("ChartsOfCharacteristicTypes");
    if !dir.exists() { return Ok(()); }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_chart_of_characteristic_types(&data) {
                objects.push(obj);
            }
        }
    }
    Ok(())
}

fn load_enums(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let dir = path.join("Enums");
    if !dir.exists() { return Ok(()); }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_enum(&data) {
                objects.push(obj);
            }
        }
    }
    Ok(())
}

fn load_constants(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let dir = path.join("Constants");
    if !dir.exists() { return Ok(()); }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_constant(&data) {
                objects.push(obj);
            }
        }
    }
    Ok(())
}

fn load_document_journals(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let dir = path.join("DocumentJournals");
    if !dir.exists() { return Ok(()); }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_document_journal(&data) {
                objects.push(obj);
            }
        }
    }
    Ok(())
}

fn load_exchange_plans(path: &Path, objects: &mut Vec<MdObject>) -> Result<(), String> {
    let dir = path.join("ExchangePlans");
    if !dir.exists() { return Ok(()); }
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
        let file_path = entry.path();
        if file_path.extension().map_or(false, |e| e == "xml") {
            let data = fs::read(&file_path).map_err(|e| e.to_string())?;
            if let Some(obj) = crate::parser::parse_exchange_plan(&data) {
                objects.push(obj);
            }
        }
    }
    Ok(())
}

fn serialize_md_type(md_type: &MdType) -> serde_json::Value {
    match md_type {
        MdType::String { length, allowed_length } => {
            serde_json::json!({
                "type": "String",
                "length": length,
                "allowed_length": allowed_length
            })
        }
        MdType::Number { digits, fraction_digits, allowed_sign } => {
            serde_json::json!({
                "type": "Number",
                "precision": digits,
                "scale": fraction_digits,
                "allowed_sign": allowed_sign
            })
        }
        MdType::Boolean => {
            serde_json::json!({"type": "Boolean"})
        }
        MdType::Date { date_fractions } => {
            serde_json::json!({
                "type": "Date",
                "date_fractions": date_fractions
            })
        }
        MdType::CatalogRef(name) => {
            serde_json::json!({"type": "CatalogRef", "ref": name})
        }
        MdType::DocumentRef(name) => {
            serde_json::json!({"type": "DocumentRef", "ref": name})
        }
        MdType::EnumRef(name) => {
            serde_json::json!({"type": "EnumRef", "ref": name})
        }
        MdType::ChartOfCharacteristicTypesRef(name) => {
            serde_json::json!({"type": "ChartOfCharacteristicTypesRef", "ref": name})
        }
        MdType::InformationRegisterRef(name) => {
            serde_json::json!({"type": "InformationRegisterRef", "ref": name})
        }
        MdType::AccumulationRegisterRef(name) => {
            serde_json::json!({"type": "AccumulationRegisterRef", "ref": name})
        }
        MdType::ExchangePlanRef(name) => {
            serde_json::json!({"type": "ExchangePlanRef", "ref": name})
        }
        MdType::Composite(types) => {
            let list: Vec<serde_json::Value> = types.iter().map(serialize_md_type).collect();
            serde_json::json!({"type": "Composite", "variants": list})
        }
    }
}

fn serialize_type_list(md_type: &MdType) -> Vec<serde_json::Value> {
    match md_type {
        MdType::Composite(types) => types.iter().map(serialize_md_type).collect(),
        _ => vec![serialize_md_type(md_type)],
    }
}

pub fn get_object(query: &str, show_attributes: bool, show_tabular: bool, project: &str) -> Result<String, String> {
    let obj = cache::find_object(project, query).ok_or_else(|| {
        format!(
            "Object '{}' not found in cache of project '{}'. Run 'v8md load-dump <path> --project {}' first.",
            query, project, project
        )
    })?;

    let mut result = serde_json::json!({
        "object": obj.full_name(),
        "type": obj.kind,
    });

    if let Some(syn) = &obj.synonym {
        result["synonym"] = serde_json::Value::String(syn.clone());
    }

    if obj.kind == "InformationRegister" {
        if let Some(p) = &obj.periodicity {
            result["periodicity"] = serde_json::Value::String(p.clone());
        }
        if let Some(wm) = &obj.write_mode {
            result["write_mode"] = serde_json::Value::String(wm.clone());
        }
        if let Some(mf) = obj.main_filter_on_period {
            result["main_filter_on_period"] = serde_json::json!(mf);
        }
    }

    if obj.kind == "AccumulationRegister" {
        if let Some(rt) = &obj.register_type {
            result["register_type"] = serde_json::Value::String(rt.clone());
        }
    }

    if obj.kind == "Constant" {
        if let Some(ct) = &obj.constant_type {
            result["constant_type"] = serialize_md_type(ct);
        }
    }

    if show_attributes || (!show_attributes && !show_tabular) {
        let mut attrs = serde_json::Map::new();

        if obj.kind == "Document" {
            if let Some(nt) = &obj.number_type {
                let mut number_info = serde_json::Map::new();
                number_info.insert("type".into(), serde_json::Value::String(nt.clone()));
                if let Some(nl) = obj.number_length {
                    number_info.insert("length".into(), serde_json::json!(nl));
                }
                if let Some(nal) = &obj.number_allowed_length {
                    number_info.insert("allowed_length".into(), serde_json::json!(nal));
                }
                attrs.insert("Номер".into(), serde_json::Value::Object(number_info));
            }
            attrs.insert(
                "Дата".into(),
                serde_json::json!({"type": "Date", "date_fractions": "DateTime"}),
            );
        }

        if obj.kind == "Catalog" {
            if let Some(ct) = &obj.code_type {
                let mut code_info = serde_json::Map::new();
                code_info.insert("type".into(), serde_json::Value::String(ct.clone()));
                if let Some(len) = obj.code_length {
                    code_info.insert("length".into(), serde_json::json!(len));
                }
                if let Some(al) = &obj.code_allowed_length {
                    code_info.insert("allowed_length".into(), serde_json::json!(al));
                }
                attrs.insert("Код".into(), serde_json::Value::Object(code_info));
            }
            if let Some(dl) = obj.description_length {
                attrs.insert(
                    "Наименование".into(),
                    serde_json::json!({
                        "type": "String",
                        "length": dl,
                        "allowed_length": "Variable"
                    }),
                );
            }
        }

        for attr in &obj.attributes {
            attrs.insert(attr.name.clone(), serialize_md_type(&attr.md_type));
        }

        if obj.kind == "InformationRegister" || obj.kind == "AccumulationRegister" {
            if !obj.dimensions.is_empty() {
                let mut dims = serde_json::Map::new();
                for dim in &obj.dimensions {
                    dims.insert(dim.name.clone(), serialize_md_type(&dim.md_type));
                }
                result["dimensions"] = serde_json::Value::Object(dims);
            }
            if !obj.resources.is_empty() {
                let mut res = serde_json::Map::new();
                for r in &obj.resources {
                    res.insert(r.name.clone(), serialize_md_type(&r.md_type));
                }
                result["resources"] = serde_json::Value::Object(res);
            }
        }

        result["properties"] = serde_json::Value::Object(attrs);
    }

    if show_tabular || (!show_attributes && !show_tabular) {
        if !obj.tabular_sections.is_empty() {
            let mut tss = serde_json::Map::new();
            for ts in &obj.tabular_sections {
                let mut ts_attrs = serde_json::Map::new();
                for attr in &ts.attributes {
                    ts_attrs.insert(attr.name.clone(), serialize_md_type(&attr.md_type));
                }
                let mut ts_obj = serde_json::Map::new();
                ts_obj.insert("columns".into(), serde_json::Value::Object(ts_attrs));
                if let Some(syn) = &ts.synonym {
                    ts_obj.insert("synonym".into(), serde_json::Value::String(syn.clone()));
                }
                tss.insert(ts.name.clone(), serde_json::Value::Object(ts_obj));
            }
            result["tabular_sections"] = serde_json::Value::Object(tss);
        }
    }

    if !obj.enum_values.is_empty() {
        let values: Vec<serde_json::Value> = obj.enum_values.iter().map(|v| {
            let mut map = serde_json::Map::new();
            map.insert("name".into(), serde_json::json!(v.name));
            if let Some(syn) = &v.synonym {
                map.insert("synonym".into(), serde_json::json!(syn));
            }
            serde_json::Value::Object(map)
        }).collect();
        result["enum_values"] = serde_json::Value::Array(values);
    }

    if !obj.journal_columns.is_empty() {
        let columns: Vec<serde_json::Value> = obj.journal_columns.iter().map(|c| {
            let mut map = serde_json::Map::new();
            map.insert("name".into(), serde_json::json!(c.name));
            if let Some(syn) = &c.synonym {
                map.insert("synonym".into(), serde_json::json!(syn));
            }
            map.insert("references".into(), serde_json::json!(c.references));
            serde_json::Value::Object(map)
        }).collect();
        result["journal_columns"] = serde_json::Value::Array(columns);
    }

    if !obj.registered_documents.is_empty() {
        result["registered_documents"] = serde_json::json!(obj.registered_documents);
    }

    if let Some(cev) = &obj.characteristic_ext_values {
        result["characteristic_ext_values"] = serde_json::json!(cev);
    }

    serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
}

pub fn get_types(query: &str, project: &str) -> Result<String, String> {
    if !cache::valid_project_name(project) {
        return Err(format!("Invalid project name: '{}'", project));
    }
    let parts: Vec<&str> = query.split('.').collect();
    if parts.len() < 3 {
        return Err(
            "Format: <Kind>.<Object>.<Attribute>, e.g. Document.Заказ.Покупатель".to_string(),
        );
    }

    let kind = parts[0];
    let obj_name = parts[1];
    let attr_name = parts[2..].join(".");

    let obj = cache::load_object(project, kind, obj_name)
        .ok_or_else(|| format!("Object '{}.{}' not found in cache of project '{}'", kind, obj_name, project))?;

    for attr in &obj.attributes {
        if attr.name == attr_name {
            let result = serde_json::json!({
                "object": obj.full_name(),
                "attribute": attr_name,
                "types": serialize_type_list(&attr.md_type),
            });
            return serde_json::to_string_pretty(&result).map_err(|e| e.to_string());
        }
    }

    for ts in &obj.tabular_sections {
        if attr_name.starts_with(&format!("{}.", ts.name)) {
            let col_name = &attr_name[ts.name.len() + 1..];
            for attr in &ts.attributes {
                if attr.name == col_name {
                    let result = serde_json::json!({
                        "object": obj.full_name(),
                        "tabular_section": ts.name,
                        "attribute": col_name,
                        "types": serialize_type_list(&attr.md_type),
                    });
                    return serde_json::to_string_pretty(&result).map_err(|e| e.to_string());
                }
            }
        }
    }

    Err(format!(
        "Attribute '{}' not found in {}",
        attr_name,
        obj.full_name()
    ))
}

pub fn list_objects(kind_filter: Option<&str>, project: &str) -> Result<String, String> {
    if !cache::valid_project_name(project) {
        return Err(format!("Invalid project name: '{}'", project));
    }
    let index = cache::load_cache_index(project)
        .ok_or_else(|| format!("Project '{}' not found. Run 'v8md load-dump <path> --project {}' first.", project, project))?;

    let mut entries: Vec<&crate::model::CacheIndexEntry> = index.objects.iter().collect();
    if let Some(filter) = kind_filter {
        entries.retain(|e| e.kind == filter);
    }

    entries.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));

    let mut output = String::new();
    for entry in &entries {
        output.push_str(&format!("{}.{}\n", entry.kind, entry.name));
    }
    output.push_str(&format!("Total: {} objects", entries.len()));

    Ok(output)
}