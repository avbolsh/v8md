use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum MdType {
    #[serde(rename = "String")]
    String { length: u32, allowed_length: String },
    #[serde(rename = "Number")]
    Number {
        digits: u32,
        fraction_digits: u32,
        allowed_sign: String,
    },
    #[serde(rename = "Boolean")]
    Boolean,
    #[serde(rename = "Date")]
    Date { date_fractions: String },
    #[serde(rename = "CatalogRef")]
    CatalogRef(String),
    #[serde(rename = "DocumentRef")]
    DocumentRef(String),
    #[serde(rename = "EnumRef")]
    EnumRef(String),
    #[serde(rename = "ChartOfCharacteristicTypesRef")]
    ChartOfCharacteristicTypesRef(String),
    #[serde(rename = "InformationRegisterRef")]
    InformationRegisterRef(String),
    #[serde(rename = "Composite")]
    Composite(Vec<MdType>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdAttribute {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synonym: Option<String>,
    pub md_type: MdType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_checking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdTabularSection {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synonym: Option<String>,
    pub attributes: Vec<MdAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdObject {
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synonym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub uuid: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub attributes: Vec<MdAttribute>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tabular_sections: Vec<MdTabularSection>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_allowed_length: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub posting: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hierarchical: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hierarchy_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_allowed_length: Option<String>,
}

impl MdObject {
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.kind, self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheIndex {
    pub config_name: String,
    pub config_version: String,
    pub objects: Vec<CacheIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheIndexEntry {
    pub kind: String,
    pub name: String,
    pub file: String,
}