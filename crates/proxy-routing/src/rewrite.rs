use crate::validator::validate_path;
use anyhow::{Result, ensure};
use std::collections::HashMap;

pub fn rewrite_path(template: &str, parameters: &HashMap<String, String>) -> Result<String> {
    let mut path = template.to_string();
    for (key, value) in parameters {
        path = path.replace(&format!("{{{key}}}"), value);
    }
    ensure!(!path.contains(['{', '}']), "unresolved rewrite parameter");
    validate_path(&path)?;
    Ok(path)
}
