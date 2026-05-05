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
    })
}