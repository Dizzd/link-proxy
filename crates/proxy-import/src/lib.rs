use anyhow::{Context, Result, bail, ensure};
use calamine::{Reader, Xlsx};
use proxy_cache::config::ImportLimits;
use proxy_domain::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
};

#[derive(Serialize, Deserialize)]
struct FlatMapping {
    external_id: String,
    mapping_key: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    source_host: String,
    #[serde(default)]
    source_path: String,
    #[serde(default = "exact")]
    path_type: String,
    #[serde(default = "yes")]
    enabled: bool,
    #[serde(default)]
    priority: i32,
    #[serde(default = "redirect")]
    default_action_type: String,
    #[serde(default)]
    default_target_url: String,
    #[serde(default)]
    default_upstream_name: String,
    #[serde(default)]
    condition_json: String,
    #[serde(default)]
    mapping_json: String,
}
fn exact() -> String {
    "exact".into()
}
fn redirect() -> String {
    "redirect".into()
}
impl FlatMapping {
    fn mapping(self, upstreams: &[Upstream]) -> Result<Mapping> {
        ensure!(!self.external_id.is_empty(), "external_id is required");
        if !self.mapping_json.is_empty() {
            let mut mapping: Mapping = serde_json::from_str(&self.mapping_json)?;
            mapping.id = self.external_id;
            return Ok(mapping);
        }
        let action = match self.default_action_type.as_str() {
            "redirect" => Action::Redirect {
                status: 302,
                target_url: self.default_target_url,
                path_handling: PathHandling::Replace,
                query_policy: QueryPolicy::Drop,
            },
            "reverse_proxy" => Action::ReverseProxy {
                upstream_id: upstreams
                    .iter()
                    .find(|u| u.name == self.default_upstream_name)
                    .context("unknown upstream name")?
                    .id
                    .clone(),
                rewrite: None,
                preserve_host: false,
            },
            _ => bail!("unknown default_action_type"),
        };
        let rules: Vec<MappingRule> = if self.condition_json.is_empty() {
            Vec::new()
        } else {
            serde_json::from_str(&self.condition_json)
                .context("condition_json must contain an array of complete mapping rules")?
        };
        Ok(Mapping {
            id: self.external_id,
            mapping_key: self.mapping_key,
            name: self.name,
            description: String::new(),
            enabled: self.enabled,
            priority: self.priority,
            source_matcher: SourceMatcher {
                hosts: self.source_host.split(';').map(str::to_owned).collect(),
                path_type: serde_json::from_value(serde_json::Value::String(self.path_type))?,
                path_value: self.source_path,
                methods: Vec::new(),
            },
            default_action: action,
            rules,
            created_at: now(),
            updated_at: now(),
        })
    }
}
pub fn error(file: &str, location: String, code: &str, description: String) -> ImportJobError {
    ImportJobError {
        file_name: file.into(),
        location,
        field: "record".into(),
        code: code.into(),
        description,
        suggestion: "Correct the record using the downloaded schema and validate again".into(),
    }
}

pub fn parse(
    bytes: &[u8],
    format: Format,
    file: &str,
    limits: &ImportLimits,
    upstreams: &[Upstream],
) -> Result<(ImportDocument, Vec<ImportJobError>)> {
    ensure!(
        bytes.len() <= limits.max_upload_bytes,
        "file exceeds upload limit"
    );
    ensure!(
        file.to_ascii_lowercase()
            .ends_with(&format!(".{}", format.as_str())),
        "extension does not match format"
    );
    let mut errors = Vec::new();
    let mut document = ImportDocument {
        domains: Vec::new(),
        upstreams: Vec::new(),
        mappings: Vec::new(),
    };
    match format {
        Format::Json => {
            json_import::parse_format(bytes, file, limits, upstreams, &mut document, &mut errors)?
        }

        Format::Csv => {
            csv_import::parse_format(bytes, file, limits, upstreams, &mut document, &mut errors)?
        }

        Format::Xlsx => {
            xlsx_import::parse_format(bytes, file, limits, upstreams, &mut document, &mut errors)?
        }
    }
    let available: HashMap<String, Upstream> = upstreams
        .iter()
        .chain(document.upstreams.iter())
        .map(|u| (u.id.clone(), u.clone()))
        .collect();
    for (index, mapping) in document.mappings.iter().enumerate() {
        if let Err(issue) = validation::validate_record(mapping, &available) {
            let location = if matches!(format, Format::Json) {
                format!("/mappings/{index}")
            } else {
                format!("mapping external_id={}", mapping.id)
            };
            errors.push(error(file, location, "invalid_mapping", issue.to_string()));
        }
    }
    ensure!(
        document.mappings.len() <= limits.max_records && errors.len() <= limits.max_records,
        "record count exceeds limit"
    );
    ensure!(
        document.domains.len() + document.upstreams.len() <= limits.max_records,
        "dependency count exceeds limit"
    );
    Ok((document, errors))
}

pub mod export;
pub use export::*;
pub mod commit;
pub use commit::merge;
pub mod job;
pub mod staging;
pub use job::*;
pub use staging::*;
mod csv_import;
mod json_import;
mod validation;
mod xlsx_import;
use validation::check_json_depth;
pub mod preview;
