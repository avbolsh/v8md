use crate::model::MdType;

pub fn parse_type_from_bytes(data: &[u8]) -> MdType {
    let text = String::from_utf8_lossy(data);

    let mut v8_types: Vec<String> = Vec::new();
    let mut string_length: Option<u32> = None;
    let mut string_allowed_length: Option<String> = None;
    let mut number_digits: Option<u32> = None;
    let mut number_fraction_digits: Option<u32> = None;
    let mut number_allowed_sign: Option<String> = None;
    let mut date_fractions: Option<String> = None;

    let mut in_string_qualifiers = false;
    let mut in_number_qualifiers = false;
    let mut in_date_qualifiers = false;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("<v8:Type>") && trimmed.ends_with("</v8:Type>") {
            let content = trimmed
                .strip_prefix("<v8:Type>")
                .unwrap()
                .strip_suffix("</v8:Type>")
                .unwrap()
                .trim();
            v8_types.push(content.to_string());
        } else if trimmed == "<v8:StringQualifiers>" {
            in_string_qualifiers = true;
        } else if trimmed == "</v8:StringQualifiers>" {
            in_string_qualifiers = false;
        } else if trimmed == "<v8:NumberQualifiers>" {
            in_number_qualifiers = true;
        } else if trimmed == "</v8:NumberQualifiers>" {
            in_number_qualifiers = false;
        } else if trimmed == "<v8:DateQualifiers>" {
            in_date_qualifiers = true;
        } else if trimmed == "</v8:DateQualifiers>" {
            in_date_qualifiers = false;
        } else if in_string_qualifiers {
            if trimmed.starts_with("<v8:Length>") {
                string_length = trimmed
                    .strip_prefix("<v8:Length>")
                    .unwrap()
                    .strip_suffix("</v8:Length>")
                    .unwrap()
                    .trim()
                    .parse()
                    .ok();
            } else if trimmed.starts_with("<v8:AllowedLength>") {
                string_allowed_length = Some(
                    trimmed
                        .strip_prefix("<v8:AllowedLength>")
                        .unwrap()
                        .strip_suffix("</v8:AllowedLength>")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
            }
        } else if in_number_qualifiers {
            if trimmed.starts_with("<v8:Digits>") {
                number_digits = trimmed
                    .strip_prefix("<v8:Digits>")
                    .unwrap()
                    .strip_suffix("</v8:Digits>")
                    .unwrap()
                    .trim()
                    .parse()
                    .ok();
            } else if trimmed.starts_with("<v8:FractionDigits>") {
                number_fraction_digits = Some(
                    trimmed
                        .strip_prefix("<v8:FractionDigits>")
                        .unwrap()
                        .strip_suffix("</v8:FractionDigits>")
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap_or(0),
                );
            } else if trimmed.starts_with("<v8:AllowedSign>") {
                number_allowed_sign = Some(
                    trimmed
                        .strip_prefix("<v8:AllowedSign>")
                        .unwrap()
                        .strip_suffix("</v8:AllowedSign>")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
            }
        } else if in_date_qualifiers {
            if trimmed.starts_with("<v8:DateFractions>") {
                date_fractions = Some(
                    trimmed
                        .strip_prefix("<v8:DateFractions>")
                        .unwrap()
                        .strip_suffix("</v8:DateFractions>")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
            }
        }
    }

    let has_xs_types = v8_types.iter().any(|t| t.starts_with("xs:"));
    let has_ref_types = v8_types.iter().any(|t| t.starts_with("cfg:"));

    if v8_types.len() == 1 && !has_xs_types {
        let t = &v8_types[0];
        if t.starts_with("cfg:") {
            if let Some(dot_pos) = t[4..].find('.') {
                let kind = &t[4..4 + dot_pos];
                let name = &t[4 + dot_pos + 1..];
                match kind {
                    "CatalogRef" => MdType::CatalogRef(name.to_string()),
                    "DocumentRef" => MdType::DocumentRef(name.to_string()),
                    "EnumRef" => MdType::EnumRef(name.to_string()),
                    "ChartOfCharacteristicTypesRef" => {
                        MdType::ChartOfCharacteristicTypesRef(name.to_string())
                    }
                    "InformationRegisterRef" => {
                        MdType::InformationRegisterRef(name.to_string())
                    }
                    "AccumulationRegisterRef" => {
                        MdType::AccumulationRegisterRef(name.to_string())
                    }
                    "ExchangePlanRef" => {
                        MdType::ExchangePlanRef(name.to_string())
                    }
                    _ => MdType::String { length: 0, allowed_length: "Variable".to_string() },
                }
            } else {
                let name = &t[4..];
                MdType::CatalogRef(name.to_string())
            }
        } else {
            MdType::String { length: 0, allowed_length: "Variable".to_string() }
        }
    } else if v8_types.is_empty() && date_fractions.is_some() && !has_ref_types {
        MdType::Date { date_fractions: date_fractions.unwrap() }
    } else if has_xs_types || has_ref_types {
        let mut types: Vec<MdType> = Vec::new();

        for t in &v8_types {
            if t.starts_with("cfg:") {
                if let Some(dot_pos) = t.find('.') {
                    let kind = &t[4..dot_pos];
                    let name = &t[dot_pos + 1..];
                    match kind {
                        "CatalogRef" => types.push(MdType::CatalogRef(name.to_string())),
                        "DocumentRef" => types.push(MdType::DocumentRef(name.to_string())),
                        "EnumRef" => types.push(MdType::EnumRef(name.to_string())),
                        "ChartOfCharacteristicTypesRef" => {
                            types.push(MdType::ChartOfCharacteristicTypesRef(name.to_string()))
                        }
                        "InformationRegisterRef" => {
                            types.push(MdType::InformationRegisterRef(name.to_string()))
                        }
                        "AccumulationRegisterRef" => {
                            types.push(MdType::AccumulationRegisterRef(name.to_string()))
                        }
                        "ExchangePlanRef" => {
                            types.push(MdType::ExchangePlanRef(name.to_string()))
                        }
                        _ => {}
                    }
                }
            } else if t == "xs:string" {
                types.push(MdType::String {
                    length: string_length.unwrap_or(0),
                    allowed_length: string_allowed_length
                        .clone()
                        .unwrap_or_else(|| "Variable".to_string()),
                });
            } else if t == "xs:decimal" {
                types.push(MdType::Number {
                    digits: number_digits.unwrap_or(0),
                    fraction_digits: number_fraction_digits.unwrap_or(0),
                    allowed_sign: number_allowed_sign
                        .clone()
                        .unwrap_or_else(|| "Any".to_string()),
                });
            } else if t == "xs:boolean" {
                types.push(MdType::Boolean);
            } else if t == "xs:dateTime" {
                types.push(MdType::Date {
                    date_fractions: date_fractions
                        .clone()
                        .unwrap_or_else(|| "DateTime".to_string()),
                });
            }
        }

        if types.len() == 1 {
            types.into_iter().next().unwrap()
        } else {
            MdType::Composite(types)
        }
    } else {
        MdType::String { length: 0, allowed_length: "Variable".to_string() }
    }
}