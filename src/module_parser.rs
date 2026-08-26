use crate::model::{BsMethod, BsModule, BsParam};

pub fn parse_bsl_module(content: &str, module_type: &str, module_name: &str) -> BsModule {
    let mut methods = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if is_method_declaration(line) {
            let mut description = None;
            let mut params_desc: Vec<(String, String, Option<String>)> = Vec::new();
            let mut return_desc = None;

            let mut comment_start = i;
            while comment_start > 0 {
                let prev = lines[comment_start - 1].trim();
                if prev.starts_with("//") {
                    comment_start -= 1;
                } else {
                    break;
                }
            }

            if comment_start < i {
                let comment_lines: Vec<&str> = lines[comment_start..i]
                    .iter()
                    .map(|l| l.trim().trim_start_matches('/').trim())
                    .collect();

                let mut in_params = false;
                let mut in_return = false;
                let mut desc_parts: Vec<String> = Vec::new();

                for cl in &comment_lines {
                    if cl.starts_with("Параметры:") || cl.starts_with("Parameters:") {
                        in_params = true;
                        in_return = false;
                        continue;
                    }
                    if cl.starts_with("Возвращаемое значение:") || cl.starts_with("Returns:") {
                        in_return = true;
                        in_params = false;
                        continue;
                    }
                    if cl.is_empty() {
                        if !in_params && !in_return {
                            if !desc_parts.is_empty() {
                                desc_parts.push(String::new());
                            }
                        }
                        continue;
                    }

                    if in_params {
                        if let Some((name, ptype, pdesc)) = parse_param_comment(cl) {
                            params_desc.push((name, ptype, pdesc));
                        } else if let Some(last) = params_desc.last_mut() {
                            if !last.2.as_ref().map_or(true, |d| d.is_empty()) {
                                last.2 = Some(format!("{} {}", last.2.as_ref().unwrap(), cl));
                            } else {
                                last.2 = Some(cl.to_string());
                            }
                        }
                    } else if in_return {
                        if return_desc.is_none() {
                            return_desc = Some(cl.to_string());
                        } else {
                            return_desc = Some(format!("{} {}", return_desc.as_ref().unwrap(), cl));
                        }
                    } else {
                        desc_parts.push(cl.to_string());
                    }
                }

                if !desc_parts.is_empty() {
                    let full_desc = desc_parts.join(" ").trim().to_string();
                    if !full_desc.is_empty() {
                        description = Some(full_desc);
                    }
                }
            }

            let (is_function, name, mut params, is_export, context_directive) =
                parse_method_signature(line);

            for p in &mut params {
                if let Some((_, ptype, pdesc)) = params_desc.iter().find(|(n, _, _)| n == &p.name) {
                    if p.param_type.is_none() && !ptype.is_empty() {
                        p.param_type = Some(ptype.clone());
                    }
                    if p.description.is_none() {
                        p.description = pdesc.clone();
                    }
                }
            }

            methods.push(BsMethod {
                name,
                is_function,
                context_directive,
                is_export,
                description,
                params,
                return_description: return_desc,
            });
        }

        i += 1;
    }

    BsModule {
        module_type: module_type.to_string(),
        module_name: module_name.to_string(),
        context: None,
        methods,
    }
}

fn is_method_declaration(line: &str) -> bool {
    let line = line.trim();
    let line = if line.starts_with("&") {
        if let Some(idx) = line.find(' ') {
            line[idx..].trim()
        } else {
            line
        }
    } else {
        line
    };
    line.starts_with("Функция ")
        || line.starts_with("Функция(")
        || line.starts_with("Function ")
        || line.starts_with("Function(")
        || line.starts_with("Процедура ")
        || line.starts_with("Процедура(")
        || line.starts_with("Procedure ")
        || line.starts_with("Procedure(")
}

fn parse_method_signature(line: &str) -> (bool, String, Vec<BsParam>, bool, Option<String>) {
    let line = line.trim();

    let (context_directive, rest) = if line.starts_with("&") {
        if let Some(idx) = line.find(' ') {
            let directive = &line[..idx];
            let rest = line[idx..].trim();
            let directive = match directive {
                "&НаСервере" | "&AtServer" => Some("&AtServer".to_string()),
                "&НаКлиенте" | "&AtClient" => Some("&AtClient".to_string()),
                "&НаСервереБезКонтекста" | "&AtServerNoContext" => {
                    Some("&AtServerNoContext".to_string())
                }
                _ => Some(directive.to_string()),
            };
            (directive, rest)
        } else {
            (None, line)
        }
    } else {
        (None, line)
    };

    let is_function = rest.starts_with("Функция ") || rest.starts_with("Function ");
    let is_procedure = rest.starts_with("Процедура ") || rest.starts_with("Procedure ");

    let rest = if is_function {
        rest.strip_prefix("Функция ")
            .or_else(|| rest.strip_prefix("Function "))
            .unwrap_or(rest)
            .trim()
    } else if is_procedure {
        rest.strip_prefix("Процедура ")
            .or_else(|| rest.strip_prefix("Procedure "))
            .unwrap_or(rest)
            .trim()
    } else {
        rest
    };

    let is_export = rest.ends_with(" Экспорт") || rest.ends_with(" Export");
    let rest = rest
        .trim_end_matches(" Экспорт")
        .trim_end_matches(" Export")
        .trim();

    let (name, params_str) = if let Some(paren_start) = rest.find('(') {
        let name = rest[..paren_start].trim().to_string();
        let params_str = &rest[paren_start..];
        (name, params_str)
    } else {
        (rest.to_string(), "()")
    };

    let params = parse_params(params_str);

    (is_function, name, params, is_export, context_directive)
}

fn parse_params(params_str: &str) -> Vec<BsParam> {
    let params_str = params_str.trim();
    if params_str == "()" || params_str.is_empty() {
        return Vec::new();
    }

    let inner = if params_str.starts_with('(') && params_str.ends_with(')') {
        &params_str[1..params_str.len() - 1]
    } else {
        params_str
    };

    if inner.trim().is_empty() {
        return Vec::new();
    }

    let parts = split_params(inner);
    let mut params = Vec::new();

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let (is_by_value, part) = if let Some(rest) = part.strip_prefix("Знач ") {
            (true, rest.trim())
        } else if let Some(rest) = part.strip_prefix("Val ") {
            (true, rest.trim())
        } else {
            (false, part)
        };

let (name, default_value) = if let Some(eq_pos) = part.find('=') {
            let (before, after) = part.split_at(eq_pos);
            let name = before.trim().to_string();
            let default = after[1..].trim().to_string();
            (name, Some(default))
        } else {
            (part.to_string(), None)
        };

        params.push(BsParam {
            name,
            is_by_value,
            default_value,
            description: None,
            param_type: None,
        });
    }

    params
}

fn split_params(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in s.chars() {
        match ch {
            '(' | '[' | '{' => {
                depth += 1;
                current.push(ch);
            }
            ')' | ']' | '}' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                result.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

fn parse_param_comment(line: &str) -> Option<(String, String, Option<String>)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.splitn(3, " - ").collect();
    match parts.len() {
        3 => Some((
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
            Some(parts[2].trim().to_string()),
        )),
        2 => Some((
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
            None,
        )),
        _ => None,
    }
}
