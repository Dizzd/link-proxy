use super::*;
pub fn spreadsheet_safe(value: &str) -> String {
    if value
        .trim_start()
        .starts_with(['=', '+', '-', '@', '\t', '\r'])
    {
        format!("'{value}")
    } else {
        value.to_string()
    }
}
const HEADERS: [&str; 14] = [
    "external_id",
    "mapping_key",
    "name",
    "source_host",
    "source_path",
    "path_type",
    "enabled",
    "priority",
    "default_action_type",
    "default_target_url",
    "default_upstream_name",
    "condition_json",
    "mapping_json",
    "schema_version",
];
fn flat_row(m: &Mapping, upstreams: &[Upstream], with_rules: bool) -> Result<Vec<String>> {
    let (action, target, upstream) = match &m.default_action {
        Action::Redirect { target_url, .. } => ("redirect", target_url.clone(), String::new()),
        Action::ReverseProxy { upstream_id, .. } => (
            "reverse_proxy",
            String::new(),
            upstreams
                .iter()
                .find(|u| u.id == *upstream_id)
                .map(|u| u.name.clone())
                .unwrap_or_default(),
        ),
    };
    let mut complete = m.clone();
    if !with_rules {
        complete.rules.clear();
    }
    Ok(vec![
        m.id.clone(),
        m.mapping_key.clone(),
        spreadsheet_safe(&m.name),
        m.source_matcher.hosts.join(";"),
        m.source_matcher.path_value.clone(),
        serde_json::to_value(m.source_matcher.path_type)?
            .as_str()
            .unwrap_or_default()
            .into(),
        m.enabled.to_string(),
        m.priority.to_string(),
        action.into(),
        target,
        upstream,
        String::new(),
        serde_json::to_string(&complete)?,
        "1".into(),
    ])
}
pub fn export(config: &Configuration, format: Format) -> Result<Vec<u8>> {
    match format {
        Format::Json => Ok(serde_json::to_vec_pretty(&ImportDocument {
            domains: config.domains.clone(),
            upstreams: config.upstreams.clone(),
            mappings: config.mappings.clone(),
        })?),
        Format::Csv => {
            let mut writer = csv::Writer::from_writer(Vec::new());
            writer.write_record(HEADERS)?;
            for m in &config.mappings {
                writer.write_record(
                    flat_row(m, &config.upstreams, true)?
                        .iter()
                        .map(|v| spreadsheet_safe(v)),
                )?;
            }
            Ok(writer.into_inner()?)
        }
        Format::Xlsx => {
            let mut workbook = rust_xlsxwriter::Workbook::new();
            let sheet = workbook.add_worksheet();
            sheet.set_name("Mappings")?;
            for (column, header) in HEADERS.iter().enumerate() {
                sheet.write_string(0, column as u16, *header)?;
            }
            for (row, m) in config.mappings.iter().enumerate() {
                for (column, value) in flat_row(m, &config.upstreams, false)?.iter().enumerate() {
                    sheet.write_string((row + 1) as u32, column as u16, value)?;
                }
            }
            let sheet = workbook.add_worksheet();
            sheet.set_name("Rules")?;
            let headers = [
                "mapping_external_id",
                "rule_id",
                "rule_name",
                "priority",
                "enabled",
                "condition_json",
                "action_json",
            ];
            for (col, header) in headers.iter().enumerate() {
                sheet.write_string(0, col as u16, *header)?;
            }
            let mut row = 1;
            for m in &config.mappings {
                for rule in &m.rules {
                    let values = [
                        m.id.clone(),
                        rule.id.clone(),
                        rule.name.clone(),
                        rule.priority.to_string(),
                        rule.enabled.to_string(),
                        serde_json::to_string(&rule.condition)?,
                        serde_json::to_string(&rule.action)?,
                    ];
                    for (col, value) in values.iter().enumerate() {
                        sheet.write_string(row, col as u16, value)?;
                    }
                    row += 1;
                }
            }
            for (name, entities) in [
                (
                    "Upstreams",
                    config
                        .upstreams
                        .iter()
                        .map(|u| Ok((u.id.clone(), serde_json::to_string(u)?)))
                        .collect::<Result<Vec<_>>>()?,
                ),
                (
                    "Domains",
                    config
                        .domains
                        .iter()
                        .map(|d| Ok((d.id.clone(), serde_json::to_string(d)?)))
                        .collect::<Result<Vec<_>>>()?,
                ),
            ] {
                let sheet = workbook.add_worksheet();
                sheet.set_name(name)?;
                sheet.write_string(0, 0, "external_id")?;
                sheet.write_string(0, 1, "data_json")?;
                for (i, (id, data)) in entities.iter().enumerate() {
                    sheet.write_string((i + 1) as u32, 0, id)?;
                    sheet.write_string((i + 1) as u32, 1, data)?;
                }
            }
            Ok(workbook.save_to_buffer()?)
        }
    }
}
pub fn template(format: Format) -> Result<Vec<u8>> {
    let mapping: Mapping = serde_json::from_value(
        serde_json::json!({"id":"example-001","mapping_key":"QR001","name":"Example mapping","source_matcher":{"hosts":["abc.example.com"],"path_type":"exact","path_value":"/QR001"},"default_action":{"type":"redirect","status":302,"target_url":"https://example.org/"}}),
    )?;
    export(
        &Configuration {
            schema_version: 1,
            mappings: vec![mapping],
            ..Default::default()
        },
        format,
    )
}
