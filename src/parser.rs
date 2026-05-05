use crate::model::*;
use crate::type_parser::parse_type_from_bytes;

fn extract_text_between(content: &str, start_tag: &str, end_tag: &str) -> Option<String> {
    let start = content.find(start_tag)?;
    let start_pos = start + start_tag.len();
    let end_pos = content[start_pos..].find(end_tag)?;
    Some(content[start_pos..start_pos + end_pos].trim().to_string())
}



fn extract_first_synonym(content: &str) -> Option<String> {
    let syn_start = content.find("<Synonym>")?;
    let syn_end = content[syn_start..].find("</Synonym>")?;
    let syn_block = &content[syn_start..syn_start + syn_end + "</Synonym>".len()];
    extract_synonym_block(syn_block)
}

fn extract_synonym_block(content: &str) -> Option<String> {
    let mut result: Option<String> = None;
    let mut pos = 0;
    while let Some(idx) = content[pos..].find("<v8:item>") {
        let item_start = pos + idx + "<v8:item>".len();
        if let Some(end_idx) = content[item_start..].find("</v8:item>") {
            let item_content = &content[item_start..item_start + end_idx];
            let lang = extract_text_between(item_content, "<v8:lang>", "</v8:lang>");
            let text = extract_text_between(item_content, "<v8:content>", "</v8:content>");
            if let (Some(l), Some(t)) = (lang, text) {
                if l == "ru" || result.is_none() {
                    result = Some(t);
                }
            }
            pos = item_start + end_idx + "</v8:item>".len();
        } else {
            break;
        }
    }
    result
}

fn parse_type_from_block(content: &str) -> Option<MdType> {
    if let Some(start) = content.find("<Type>") {
        let type_start = start + "<Type>".len();
        if let Some(offset) = content[type_start..].find("</Type>") {
            let type_content = &content[type_start..type_start + offset];
            return Some(parse_type_from_bytes(type_content.as_bytes()));
        }
    }
    None
}

fn parse_attribute_block(content: &str) -> Option<MdAttribute> {
    let name = extract_text_between(content, "<Name>", "</Name>")?;
    let syn_start = content.find("<Synonym>");
    let synonym = syn_start.and_then(|s| {
        let block_end = content[s..].find("</Synonym>")?;
        let syn_block = &content[s..s + block_end + "</Synonym>".len()];
        extract_synonym_block(syn_block)
    });
    let md_type = parse_type_from_block(content).unwrap_or(MdType::String {
        length: 0,
        allowed_length: "Variable".to_string(),
    });
    let fill_checking = extract_text_between(content, "<FillChecking>", "</FillChecking>");
    let indexing = extract_text_between(content, "<Indexing>", "</Indexing>");
    let use_scope = extract_text_between(content, "<Use>", "</Use>");

    Some(MdAttribute {
        name,
        synonym,
        md_type,
        fill_checking,
        indexing,
        use_scope,
    })
}

fn parse_tabular_section_block(content: &str) -> Option<MdTabularSection> {
    let name = extract_text_between(content, "<Name>", "</Name>")?;
    let syn_start = content.find("<Synonym>");
    let synonym = syn_start.and_then(|s| {
        let block_end = content[s..].find("</Synonym>")?;
        let syn_block = &content[s..s + block_end + "</Synonym>".len()];
        extract_synonym_block(syn_block)
    });

    let mut attributes = Vec::new();
    let mut pos = 0;
    while pos < content.len() {
        if let Some(attr_idx) = content[pos..].find("<Attribute uuid=\"") {
            let abs_start = pos + attr_idx;
            if let Some(attr_end_offset) = content[abs_start..].find("</Attribute>") {
                let block = &content[abs_start..abs_start + attr_end_offset + "</Attribute>".len()];
                if let Some(attr) = parse_attribute_block(block) {
                    attributes.push(attr);
                }
                pos = abs_start + attr_end_offset + "</Attribute>".len();
                continue;
            }
        }
        break;
    }

    Some(MdTabularSection {
        name,
        synonym,
        attributes,
    })
}

fn find_matching_close(content: &str, start_pos: usize, tag_name: &str) -> Option<usize> {
    let open_tag = format!("<{}>", tag_name);
    let close_tag = format!("</{}>", tag_name);
    let mut depth = 1;
    let mut pos = start_pos;
    while depth > 0 {
        let next_open = content[pos..].find(&open_tag).map(|i| pos + i);
        let next_close = content[pos..].find(&close_tag).map(|i| pos + i);
        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                pos = o + open_tag.len();
            }
            (_, Some(c)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(c);
                }
                pos = c + close_tag.len();
            }
            _ => return None,
        }
    }
    None
}

fn extract_properties_block(content: &str) -> Option<&str> {
    let props_start = content.find("<Properties>")?;
    let props_end = find_matching_close(content, props_start + "<Properties>".len(), "Properties")?;
    Some(&content[props_start..props_end + "</Properties>".len()])
}

pub fn parse_document(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);

    let uuid = {
        let start = text.find("<Document uuid=\"")? + "<Document uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };

    let name = extract_text_between(&text, "<Name>", "</Name>")?;

    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());

    let number_type = extract_text_between(props, "<NumberType>", "</NumberType>");
    let number_length = extract_text_between(props, "<NumberLength>", "</NumberLength>")
        .and_then(|s| s.parse().ok());
    let number_allowed_length =
        extract_text_between(props, "<NumberAllowedLength>", "</NumberAllowedLength>");
    let posting = extract_text_between(props, "<Posting>", "</Posting>");

    let mut attributes = Vec::new();
    let mut tabular_sections = Vec::new();

    let child_start = text.find("<ChildObjects>")?;
    let child_end_pos = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects")?;
    let child_content_doc = &text[child_start..child_end_pos + "</ChildObjects>".len()];

    let mut ts_ranges: Vec<(usize, usize)> = Vec::new();
    let mut search_pos = 0;
    while let Some(ts_idx) = child_content_doc[search_pos..].find("<TabularSection uuid=\"") {
        let abs_start = search_pos + ts_idx;
        if let Some(ts_end_offset) = child_content_doc[abs_start..].find("</TabularSection>") {
            let abs_end = abs_start + ts_end_offset + "</TabularSection>".len();
            ts_ranges.push((abs_start, abs_end));
            search_pos = abs_end;
        } else {
            break;
        }
    }

    for &(start, end) in &ts_ranges {
        let block = &child_content_doc[start..end];
        if let Some(ts) = parse_tabular_section_block(block) {
            tabular_sections.push(ts);
        }
    }

    let mut pos = 0;
    while pos < child_content_doc.len() {
        if let Some(attr_idx) = child_content_doc[pos..].find("<Attribute uuid=\"") {
            let abs_start = pos + attr_idx;
            if let Some(attr_end_offset) = child_content_doc[abs_start..].find("</Attribute>") {
                let abs_end = abs_start + attr_end_offset + "</Attribute>".len();
                let inside_ts = ts_ranges.iter().any(|&(ts_start, ts_end)| abs_start >= ts_start && abs_end <= ts_end);
                if !inside_ts {
                    let block = &child_content_doc[abs_start..abs_end];
                    if let Some(attr) = parse_attribute_block(block) {
                        attributes.push(attr);
                    }
                }
                pos = abs_end;
                continue;
            }
        }
        break;
    }

    Some(MdObject {
        kind: "Document".to_string(),
        name,
        synonym,
        comment,
        uuid,
        attributes,
        tabular_sections,
        number_type,
        number_length,
        number_allowed_length,
        posting,
        hierarchical: None,
        hierarchy_type: None,
        code_length: None,
        description_length: None,
        code_type: None,
        code_allowed_length: None,
        dimensions: Vec::new(),
        resources: Vec::new(),
        periodicity: None,
        write_mode: None,
        main_filter_on_period: None,
        enum_values: Vec::new(),
        journal_columns: Vec::new(),
        registered_documents: Vec::new(),
        register_type: None,
        constant_type: None,
        characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_catalog(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);

    let uuid = {
        let start = text.find("<Catalog uuid=\"")? + "<Catalog uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };

    let name = extract_text_between(&text, "<Name>", "</Name>")?;

    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());

    let hierarchical = extract_text_between(props, "<Hierarchical>", "</Hierarchical>")
        .and_then(|s| s.parse().ok());
    let hierarchy_type = extract_text_between(props, "<HierarchyType>", "</HierarchyType>");
    let code_length = extract_text_between(props, "<CodeLength>", "</CodeLength>")
        .and_then(|s| s.parse().ok());
    let description_length =
        extract_text_between(props, "<DescriptionLength>", "</DescriptionLength>")
            .and_then(|s| s.parse().ok());
    let code_type = extract_text_between(props, "<CodeType>", "</CodeType>");
    let code_allowed_length =
        extract_text_between(props, "<CodeAllowedLength>", "</CodeAllowedLength>");

    let mut attributes = Vec::new();
    let mut tabular_sections = Vec::new();

    let child_start = text.find("<ChildObjects>")?;
    let child_end_pos = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects")?;
    let child_content_cat = &text[child_start..child_end_pos + "</ChildObjects>".len()];

    let mut ts_ranges: Vec<(usize, usize)> = Vec::new();
    let mut search_pos = 0;
    while let Some(ts_idx) = child_content_cat[search_pos..].find("<TabularSection uuid=\"") {
        let abs_start = search_pos + ts_idx;
        if let Some(ts_end_offset) = child_content_cat[abs_start..].find("</TabularSection>") {
            let abs_end = abs_start + ts_end_offset + "</TabularSection>".len();
            ts_ranges.push((abs_start, abs_end));
            search_pos = abs_end;
        } else {
            break;
        }
    }

    for &(start, end) in &ts_ranges {
        let block = &child_content_cat[start..end];
        if let Some(ts) = parse_tabular_section_block(block) {
            tabular_sections.push(ts);
        }
    }

    let mut pos = 0;
    while pos < child_content_cat.len() {
        if let Some(attr_idx) = child_content_cat[pos..].find("<Attribute uuid=\"") {
            let abs_start = pos + attr_idx;
            if let Some(attr_end_offset) = child_content_cat[abs_start..].find("</Attribute>") {
                let abs_end = abs_start + attr_end_offset + "</Attribute>".len();
                let inside_ts = ts_ranges.iter().any(|&(ts_start, ts_end)| abs_start >= ts_start && abs_end <= ts_end);
                if !inside_ts {
                    let block = &child_content_cat[abs_start..abs_end];
                    if let Some(attr) = parse_attribute_block(block) {
                        attributes.push(attr);
                    }
                }
                pos = abs_end;
                continue;
            }
        }
        break;
    }

    Some(MdObject {
        kind: "Catalog".to_string(),
        name,
        synonym,
        comment,
        uuid,
        attributes,
        tabular_sections,
        number_type: None,
        number_length: None,
        number_allowed_length: None,
        posting: None,
        hierarchical,
        hierarchy_type,
        code_length,
        description_length,
        code_type,
        code_allowed_length,
        dimensions: Vec::new(),
        resources: Vec::new(),
        periodicity: None,
        write_mode: None,
        main_filter_on_period: None,
        enum_values: Vec::new(),
        journal_columns: Vec::new(),
        registered_documents: Vec::new(),
        register_type: None,
        constant_type: None,
        characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_information_register(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);

    let uuid = {
        let start = text.find("<InformationRegister uuid=\"")? + "<InformationRegister uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };

    let name = extract_text_between(&text, "<Name>", "</Name>")?;

    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());

    let periodicity = extract_text_between(props, "<InformationRegisterPeriodicity>", "</InformationRegisterPeriodicity>");
    let write_mode = extract_text_between(props, "<WriteMode>", "</WriteMode>");
    let main_filter_on_period = extract_text_between(props, "<MainFilterOnPeriod>", "</MainFilterOnPeriod>")
        .and_then(|s| s.parse().ok());

    let mut dimensions = Vec::new();
    let mut resources = Vec::new();
    let mut attributes = Vec::new();

    let child_start = text.find("<ChildObjects>")?;
    let child_end_pos = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects")?;
    let child_content = &text[child_start..child_end_pos + "</ChildObjects>".len()];

    let mut pos = 0;
    while pos < child_content.len() {
        let dim_idx = child_content[pos..].find("<Dimension uuid=\"").map(|i| pos + i);
        let res_idx = child_content[pos..].find("<Resource uuid=\"").map(|i| pos + i);
        let attr_idx = child_content[pos..].find("<Attribute uuid=\"").map(|i| pos + i);

        let next = match (dim_idx, res_idx, attr_idx) {
            (Some(d), Some(r), Some(a)) => Some(d.min(r).min(a)),
            (Some(d), Some(r), None) => Some(d.min(r)),
            (Some(d), None, Some(a)) => Some(d.min(a)),
            (None, Some(r), Some(a)) => Some(r.min(a)),
            (Some(d), None, None) => Some(d),
            (None, Some(r), None) => Some(r),
            (None, None, Some(a)) => Some(a),
            (None, None, None) => None,
        };

        if let Some(start) = next {
            let tag = if Some(start) == dim_idx {
                "Dimension"
            } else if Some(start) == res_idx {
                "Resource"
            } else {
                "Attribute"
            };

            let close_tag = format!("</{}>", tag);
            if let Some(end_offset) = child_content[start..].find(&close_tag) {
                let abs_end = start + end_offset + close_tag.len();
                let block = &child_content[start..abs_end];
                if let Some(attr) = parse_attribute_block(block) {
                    match tag {
                        "Dimension" => dimensions.push(attr),
                        "Resource" => resources.push(attr),
                        "Attribute" => attributes.push(attr),
                        _ => {}
                    }
                }
                pos = abs_end;
                continue;
            }
        }
        break;
    }

    Some(MdObject {
        kind: "InformationRegister".to_string(),
        name,
        synonym,
        comment,
        uuid,
        attributes,
        tabular_sections: Vec::new(),
        number_type: None,
        number_length: None,
        number_allowed_length: None,
        posting: None,
        hierarchical: None,
        hierarchy_type: None,
        code_length: None,
        description_length: None,
        code_type: None,
        code_allowed_length: None,
        dimensions,
        resources,
        periodicity,
        write_mode,
        main_filter_on_period,
        enum_values: Vec::new(),
        journal_columns: Vec::new(),
        registered_documents: Vec::new(),
        register_type: None,
        constant_type: None,
        characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_accumulation_register(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);
    let uuid = {
        let start = text.find("<AccumulationRegister uuid=\"")? + "<AccumulationRegister uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };
    let name = extract_text_between(&text, "<Name>", "</Name>")?;
    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());
    let register_type = extract_text_between(props, "<RegisterType>", "</RegisterType>");

    let mut dimensions = Vec::new();
    let mut resources = Vec::new();
    let child_start = text.find("<ChildObjects>")?;
    let child_end_pos = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects")?;
    let child_content = &text[child_start..child_end_pos + "</ChildObjects>".len()];

    let mut pos = 0;
    while pos < child_content.len() {
        let dim_idx = child_content[pos..].find("<Dimension uuid=\"").map(|i| pos + i);
        let res_idx = child_content[pos..].find("<Resource uuid=\"").map(|i| pos + i);
        let next = match (dim_idx, res_idx) {
            (Some(d), Some(r)) => Some(d.min(r)),
            (Some(d), None) => Some(d),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        };
        if let Some(start) = next {
            let tag = if Some(start) == dim_idx { "Dimension" } else { "Resource" };
            let close_tag = format!("</{}>", tag);
            if let Some(end_offset) = child_content[start..].find(&close_tag) {
                let abs_end = start + end_offset + close_tag.len();
                let block = &child_content[start..abs_end];
                if let Some(attr) = parse_attribute_block(block) {
                    match tag {
                        "Dimension" => dimensions.push(attr),
                        "Resource" => resources.push(attr),
                        _ => {}
                    }
                }
                pos = abs_end;
                continue;
            }
        }
        break;
    }

    Some(MdObject {
        kind: "AccumulationRegister".to_string(),
        name, synonym, comment, uuid,
        attributes: Vec::new(), tabular_sections: Vec::new(),
        number_type: None, number_length: None, number_allowed_length: None, posting: None,
        hierarchical: None, hierarchy_type: None,
        code_length: None, description_length: None, code_type: None, code_allowed_length: None,
        dimensions, resources,
        periodicity: None, write_mode: None, main_filter_on_period: None,
        enum_values: Vec::new(), journal_columns: Vec::new(), registered_documents: Vec::new(),
        register_type, constant_type: None, characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_chart_of_characteristic_types(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);
    let uuid = {
        let start = text.find("<ChartOfCharacteristicTypes uuid=\"")? + "<ChartOfCharacteristicTypes uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };
    let name = extract_text_between(&text, "<Name>", "</Name>")?;
    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());
    let hierarchical = extract_text_between(props, "<Hierarchical>", "</Hierarchical>").and_then(|s| s.parse().ok());
    let code_length = extract_text_between(props, "<CodeLength>", "</CodeLength>").and_then(|s| s.parse().ok());
    let description_length = extract_text_between(props, "<DescriptionLength>", "</DescriptionLength>").and_then(|s| s.parse().ok());
    let code_allowed_length = extract_text_between(props, "<CodeAllowedLength>", "</CodeAllowedLength>");
    let characteristic_ext_values = extract_text_between(props, "<CharacteristicExtValues>", "</CharacteristicExtValues>");

    let mut attributes = Vec::new();
    let mut tabular_sections = Vec::new();
    let child_start = text.find("<ChildObjects>")?;
    let child_end_pos = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects")?;
    let child_content = &text[child_start..child_end_pos + "</ChildObjects>".len()];

    let mut ts_ranges: Vec<(usize, usize)> = Vec::new();
    let mut search_pos = 0;
    while let Some(ts_idx) = child_content[search_pos..].find("<TabularSection uuid=\"") {
        let abs_start = search_pos + ts_idx;
        if let Some(ts_end_offset) = child_content[abs_start..].find("</TabularSection>") {
            let abs_end = abs_start + ts_end_offset + "</TabularSection>".len();
            ts_ranges.push((abs_start, abs_end));
            search_pos = abs_end;
        } else { break; }
    }
    for &(start, end) in &ts_ranges {
        let block = &child_content[start..end];
        if let Some(ts) = parse_tabular_section_block(block) { tabular_sections.push(ts); }
    }
    let mut pos = 0;
    while pos < child_content.len() {
        if let Some(attr_idx) = child_content[pos..].find("<Attribute uuid=\"") {
            let abs_start = pos + attr_idx;
            if let Some(attr_end_offset) = child_content[abs_start..].find("</Attribute>") {
                let abs_end = abs_start + attr_end_offset + "</Attribute>".len();
                let inside_ts = ts_ranges.iter().any(|&(ts_start, ts_end)| abs_start >= ts_start && abs_end <= ts_end);
                if !inside_ts {
                    let block = &child_content[abs_start..abs_end];
                    if let Some(attr) = parse_attribute_block(block) { attributes.push(attr); }
                }
                pos = abs_end;
                continue;
            }
        }
        break;
    }

    Some(MdObject {
        kind: "ChartOfCharacteristicTypes".to_string(),
        name, synonym, comment, uuid,
        attributes, tabular_sections,
        number_type: None, number_length: None, number_allowed_length: None, posting: None,
        hierarchical, hierarchy_type: None,
        code_length, description_length, code_type: None, code_allowed_length,
        dimensions: Vec::new(), resources: Vec::new(),
        periodicity: None, write_mode: None, main_filter_on_period: None,
        enum_values: Vec::new(), journal_columns: Vec::new(), registered_documents: Vec::new(),
        register_type: None, constant_type: None, characteristic_ext_values,
        modules: Vec::new(),
    })
}

pub fn parse_enum(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);
    let uuid = {
        let start = text.find("<Enum uuid=\"")? + "<Enum uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };
    let name = extract_text_between(&text, "<Name>", "</Name>")?;
    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());

    let mut enum_values = Vec::new();
    let child_start = text.find("<ChildObjects>")?;
    let child_end_pos = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects")?;
    let child_content = &text[child_start..child_end_pos + "</ChildObjects>".len()];
    let mut pos = 0;
    while let Some(idx) = child_content[pos..].find("<EnumValue uuid=\"") {
        let abs_start = pos + idx;
        if let Some(end_offset) = child_content[abs_start..].find("</EnumValue>") {
            let abs_end = abs_start + end_offset + "</EnumValue>".len();
            let block = &child_content[abs_start..abs_end];
            if let Some(val_name) = extract_text_between(block, "<Name>", "</Name>") {
                let syn_start = block.find("<Synonym>");
                let val_synonym = syn_start.and_then(|s| {
                    let block_end = block[s..].find("</Synonym>")?;
                    let syn_block = &block[s..s + block_end + "</Synonym>".len()];
                    extract_synonym_block(syn_block)
                });
                enum_values.push(MdEnumValue { name: val_name, synonym: val_synonym });
            }
            pos = abs_end;
        } else { break; }
    }

    Some(MdObject {
        kind: "Enum".to_string(),
        name, synonym, comment, uuid,
        attributes: Vec::new(), tabular_sections: Vec::new(),
        number_type: None, number_length: None, number_allowed_length: None, posting: None,
        hierarchical: None, hierarchy_type: None,
        code_length: None, description_length: None, code_type: None, code_allowed_length: None,
        dimensions: Vec::new(), resources: Vec::new(),
        periodicity: None, write_mode: None, main_filter_on_period: None,
        enum_values, journal_columns: Vec::new(), registered_documents: Vec::new(),
        register_type: None, constant_type: None, characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_constant(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);
    let uuid = {
        let start = text.find("<Constant uuid=\"")? + "<Constant uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };
    let name = extract_text_between(&text, "<Name>", "</Name>")?;
    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());
    let constant_type = parse_type_from_block(props);

    Some(MdObject {
        kind: "Constant".to_string(),
        name, synonym, comment, uuid,
        attributes: Vec::new(), tabular_sections: Vec::new(),
        number_type: None, number_length: None, number_allowed_length: None, posting: None,
        hierarchical: None, hierarchy_type: None,
        code_length: None, description_length: None, code_type: None, code_allowed_length: None,
        dimensions: Vec::new(), resources: Vec::new(),
        periodicity: None, write_mode: None, main_filter_on_period: None,
        enum_values: Vec::new(), journal_columns: Vec::new(), registered_documents: Vec::new(),
        register_type: None, constant_type, characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_document_journal(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);
    let uuid = {
        let start = text.find("<DocumentJournal uuid=\"")? + "<DocumentJournal uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };
    let name = extract_text_between(&text, "<Name>", "</Name>")?;
    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());

    let mut registered_documents = Vec::new();
    let mut reg_pos = 0;
    while let Some(idx) = props[reg_pos..].find("<xr:Item xsi:type=\"xr:MDObjectRef\">") {
        let item_start = reg_pos + idx + "<xr:Item xsi:type=\"xr:MDObjectRef\">".len();
        if let Some(end_idx) = props[item_start..].find("</xr:Item>") {
            registered_documents.push(props[item_start..item_start + end_idx].trim().to_string());
            reg_pos = item_start + end_idx + "</xr:Item>".len();
        } else { break; }
    }

    let mut journal_columns = Vec::new();
    let child_start = text.find("<ChildObjects>")?;
    let child_end_pos = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects")?;
    let child_content = &text[child_start..child_end_pos + "</ChildObjects>".len()];
    let mut pos = 0;
    while let Some(idx) = child_content[pos..].find("<Column uuid=\"") {
        let abs_start = pos + idx;
        if let Some(end_offset) = child_content[abs_start..].find("</Column>") {
            let abs_end = abs_start + end_offset + "</Column>".len();
            let block = &child_content[abs_start..abs_end];
            if let Some(col_name) = extract_text_between(block, "<Name>", "</Name>") {
                let syn_start = block.find("<Synonym>");
                let col_synonym = syn_start.and_then(|s| {
                    let block_end = block[s..].find("</Synonym>")?;
                    let syn_block = &block[s..s + block_end + "</Synonym>".len()];
                    extract_synonym_block(syn_block)
                });
                let mut references = Vec::new();
                let mut ref_pos = 0;
                while let Some(ref_idx) = block[ref_pos..].find("<xr:Item xsi:type=\"xr:MDObjectRef\">") {
                    let ref_start = ref_pos + ref_idx + "<xr:Item xsi:type=\"xr:MDObjectRef\">".len();
                    if let Some(ref_end) = block[ref_start..].find("</xr:Item>") {
                        references.push(block[ref_start..ref_start + ref_end].trim().to_string());
                        ref_pos = ref_start + ref_end + "</xr:Item>".len();
                    } else { break; }
                }
                journal_columns.push(MdJournalColumn { name: col_name, synonym: col_synonym, references });
            }
            pos = abs_end;
        } else { break; }
    }

    Some(MdObject {
        kind: "DocumentJournal".to_string(),
        name, synonym, comment, uuid,
        attributes: Vec::new(), tabular_sections: Vec::new(),
        number_type: None, number_length: None, number_allowed_length: None, posting: None,
        hierarchical: None, hierarchy_type: None,
        code_length: None, description_length: None, code_type: None, code_allowed_length: None,
        dimensions: Vec::new(), resources: Vec::new(),
        periodicity: None, write_mode: None, main_filter_on_period: None,
        enum_values: Vec::new(), journal_columns, registered_documents,
        register_type: None, constant_type: None, characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_exchange_plan(xml: &[u8]) -> Option<MdObject> {
    let text = String::from_utf8_lossy(xml);
    let uuid = {
        let start = text.find("<ExchangePlan uuid=\"")? + "<ExchangePlan uuid=\"".len();
        let end = text[start..].find('"')?;
        text[start..start + end].to_string()
    };
    let name = extract_text_between(&text, "<Name>", "</Name>")?;
    let props = extract_properties_block(&text)?;
    let synonym = extract_first_synonym(props);
    let comment = extract_text_between(props, "<Comment>", "</Comment>").filter(|c| !c.is_empty());
    let code_length = extract_text_between(props, "<CodeLength>", "</CodeLength>").and_then(|s| s.parse().ok());
    let description_length = extract_text_between(props, "<DescriptionLength>", "</DescriptionLength>").and_then(|s| s.parse().ok());

    let mut attributes = Vec::new();
    let mut tabular_sections = Vec::new();
    if let Some(child_start) = text.find("<ChildObjects>") {
        if let Some(child_end_pos) = find_matching_close(&text, child_start + "<ChildObjects>".len(), "ChildObjects") {
            let child_content = &text[child_start..child_end_pos + "</ChildObjects>".len()];
            let mut ts_ranges: Vec<(usize, usize)> = Vec::new();
            let mut search_pos = 0;
            while let Some(ts_idx) = child_content[search_pos..].find("<TabularSection uuid=\"") {
                let abs_start = search_pos + ts_idx;
                if let Some(ts_end_offset) = child_content[abs_start..].find("</TabularSection>") {
                    let abs_end = abs_start + ts_end_offset + "</TabularSection>".len();
                    ts_ranges.push((abs_start, abs_end));
                    search_pos = abs_end;
                } else { break; }
            }
            for &(start, end) in &ts_ranges {
                let block = &child_content[start..end];
                if let Some(ts) = parse_tabular_section_block(block) { tabular_sections.push(ts); }
            }
            let mut pos = 0;
            while pos < child_content.len() {
                if let Some(attr_idx) = child_content[pos..].find("<Attribute uuid=\"") {
                    let abs_start = pos + attr_idx;
                    if let Some(attr_end_offset) = child_content[abs_start..].find("</Attribute>") {
                        let abs_end = abs_start + attr_end_offset + "</Attribute>".len();
                        let inside_ts = ts_ranges.iter().any(|&(ts_start, ts_end)| abs_start >= ts_start && abs_end <= ts_end);
                        if !inside_ts {
                            let block = &child_content[abs_start..abs_end];
                            if let Some(attr) = parse_attribute_block(block) { attributes.push(attr); }
                        }
                        pos = abs_end;
                        continue;
                    }
                }
                break;
            }
        }
    }

    Some(MdObject {
        kind: "ExchangePlan".to_string(),
        name, synonym, comment, uuid,
        attributes, tabular_sections,
        number_type: None, number_length: None, number_allowed_length: None, posting: None,
        hierarchical: None, hierarchy_type: None,
        code_length, description_length, code_type: None, code_allowed_length: None,
        dimensions: Vec::new(), resources: Vec::new(),
        periodicity: None, write_mode: None, main_filter_on_period: None,
        enum_values: Vec::new(), journal_columns: Vec::new(), registered_documents: Vec::new(),
        register_type: None, constant_type: None, characteristic_ext_values: None,
        modules: Vec::new(),
    })
}

pub fn parse_common_module_xml(xml: &[u8]) -> Option<BsContext> {
    let text = String::from_utf8_lossy(xml);
    let props = extract_properties_block(&text)?;

    let parse_bool = |tag: &str| -> bool {
        extract_text_between(props, &format!("<{}>", tag), &format!("</{}>", tag))
            .map(|v| v == "true")
            .unwrap_or(false)
    };

    Some(BsContext {
        server: parse_bool("Server"),
        server_call: parse_bool("ServerCall"),
        client_managed_app: parse_bool("ClientManagedApplication"),
        client_ordinary_app: parse_bool("ClientOrdinaryApplication"),
        external_connection: parse_bool("ExternalConnection"),
        global: parse_bool("Global"),
        privileged: parse_bool("Privileged"),
    })
}