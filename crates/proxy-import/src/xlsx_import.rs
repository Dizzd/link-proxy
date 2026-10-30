use super::*;
pub(crate) fn parse_format(
    bytes: &[u8],
    file: &str,
    limits: &ImportLimits,
    upstreams: &[Upstream],
    document: &mut ImportDocument,
    errors: &mut Vec<ImportJobError>,
) -> Result<()> {
    ensure!(
        bytes.starts_with(b"PK\x03\x04"),
        "invalid XLSX ZIP signature"
    );
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))?;
    ensure!(zip.len() <= 512, "too many XLSX archive entries");
    let mut expanded = 0_u64;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        ensure!(entry.enclosed_name().is_some(), "unsafe XLSX archive path");
        expanded = expanded
            .checked_add(entry.size())
            .context("XLSX size overflow")?;
        ensure!(
            expanded <= limits.max_uncompressed_bytes,
            "XLSX decompression limit exceeded"
        );
        // Verify actual decompression rather than trusting ZIP central-directory sizes.
        let mut bounded = std::io::Read::take(&mut entry, limits.max_uncompressed_bytes + 1);
        let actual = std::io::copy(&mut bounded, &mut std::io::sink())?;
        ensure!(actual == entry.size(), "XLSX entry size mismatch");
    }
    let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(bytes))?;
    ensure!(
        workbook.sheet_names().iter().any(|n| n == "Mappings"),
        "XLSX requires Mappings sheet"
    );
    // Dependency sheets use stable external IDs and full versioned entity JSON.
    if workbook.sheet_names().iter().any(|n| n == "Upstreams") {
        let range = workbook.worksheet_range("Upstreams")?;
        ensure!(
            range.height() <= limits.max_records + 1 && range.width() <= 2,
            "Upstreams dimensions exceed limits"
        );
        for row in range.rows().skip(1) {
            let data = row.get(1).context("Upstreams data missing")?.to_string();
            ensure!(
                data.len() <= limits.max_record_bytes,
                "Upstream record exceeds limit"
            );
            let entity: Upstream = serde_json::from_str(&data)?;
            ensure!(
                row.first().map(ToString::to_string).as_deref() == Some(entity.id.as_str()),
                "upstream external ID mismatch"
            );
            document.upstreams.push(entity);
        }
    }
    if workbook.sheet_names().iter().any(|n| n == "Domains") {
        let range = workbook.worksheet_range("Domains")?;
        ensure!(
            range.height() <= limits.max_records + 1 && range.width() <= 2,
            "Domains dimensions exceed limits"
        );
        for row in range.rows().skip(1) {
            let data = row.get(1).context("Domains data missing")?.to_string();
            ensure!(
                data.len() <= limits.max_record_bytes,
                "Domain record exceeds limit"
            );
            let entity: Domain = serde_json::from_str(&data)?;
            ensure!(
                row.first().map(ToString::to_string).as_deref() == Some(entity.id.as_str()),
                "domain external ID mismatch"
            );
            document.domains.push(entity);
        }
    }
    let available_upstreams: Vec<_> = upstreams
        .iter()
        .chain(document.upstreams.iter())
        .cloned()
        .collect();
    let range = workbook.worksheet_range("Mappings")?;
    ensure!(
        range.height() <= limits.max_records + 1 && range.width() <= 32,
        "XLSX sheet dimensions exceed limits"
    );
    let mut rows = range.rows();
    let headers: Vec<String> = rows
        .next()
        .context("Mappings header missing")?
        .iter()
        .map(ToString::to_string)
        .collect();
    ensure!(
        headers.iter().any(|h| h == "mapping_key"),
        "missing header: mapping_key"
    );
    for (i, row) in rows.enumerate() {
        let values: HashMap<&str, String> = headers
            .iter()
            .zip(row)
            .map(|(h, v)| (h.as_str(), v.to_string()))
            .collect();
        let record_bytes = values.values().map(String::len).sum::<usize>();
        let parsed: Result<Mapping> = (|| {
            ensure!(
                record_bytes <= limits.max_record_bytes,
                "XLSX record too large"
            );
            ensure!(
                !row.iter().any(|v| matches!(v, calamine::Data::Error(_))),
                "XLSX error cells are not accepted"
            );
            let field = |name: &str| values.get(name).cloned().unwrap_or_default();
            FlatMapping {
                external_id: field("external_id"),
                mapping_key: field("mapping_key"),
                name: field("name"),
                source_host: field("source_host"),
                source_path: field("source_path"),
                path_type: if field("path_type").is_empty() {
                    exact()
                } else {
                    field("path_type")
                },
                enabled: !matches!(
                    field("enabled").to_ascii_lowercase().as_str(),
                    "false" | "0"
                ),
                priority: if field("priority").is_empty() {
                    0
                } else {
                    field("priority").parse()?
                },
                default_action_type: if field("default_action_type").is_empty() {
                    redirect()
                } else {
                    field("default_action_type")
                },
                default_target_url: field("default_target_url"),
                default_upstream_name: field("default_upstream_name"),
                condition_json: field("condition_json"),
                mapping_json: field("mapping_json"),
            }
            .mapping(&available_upstreams)
        })();
        match parsed {
            Ok(m) => document.mappings.push(m),
            Err(e) => errors.push(error(
                file,
                format!("Mappings!{}", i + 2),
                "invalid_record",
                e.to_string(),
            )),
        }
    }
    if workbook.sheet_names().iter().any(|n| n == "Rules") {
        let rules = workbook.worksheet_range("Rules")?;
        ensure!(
            rules.height() <= limits.max_records * 16 + 1 && rules.width() <= 16,
            "Rules sheet exceeds limits"
        );
        let mut rows = rules.rows();
        let headers: Vec<String> = rows
            .next()
            .context("Rules header missing")?
            .iter()
            .map(ToString::to_string)
            .collect();
        for required in ["mapping_external_id", "condition_json", "action_json"] {
            ensure!(
                headers.iter().any(|h| h == required),
                "missing Rules header: {required}"
            );
        }
        let index: HashMap<String, usize> = document
            .mappings
            .iter()
            .enumerate()
            .map(|(i, m)| (m.id.clone(), i))
            .collect();
        for (i, row) in rows.enumerate() {
            let values: HashMap<&str, String> = headers
                .iter()
                .zip(row)
                .map(|(h, v)| (h.as_str(), v.to_string()))
                .collect();
            let field = |name: &str| values.get(name).cloned().unwrap_or_default();
            let parsed: Result<()> = (|| {
                ensure!(
                    values.values().map(String::len).sum::<usize>() <= limits.max_record_bytes,
                    "rule row too large"
                );
                let ordinal = *index
                    .get(&field("mapping_external_id"))
                    .context("unknown mapping_external_id")?;
                let rule = MappingRule {
                    id: if field("rule_id").is_empty() {
                        new_id()
                    } else {
                        field("rule_id")
                    },
                    name: field("rule_name"),
                    priority: field("priority").parse()?,
                    enabled: !matches!(field("enabled").as_str(), "false" | "0"),
                    condition: serde_json::from_str(&field("condition_json"))?,
                    action: serde_json::from_str(&field("action_json"))?,
                    condition_schema_version: 1,
                };
                document.mappings[ordinal].rules.push(rule);
                Ok(())
            })();
            if let Err(e) = parsed {
                errors.push(error(
                    file,
                    format!("Rules!{}", i + 2),
                    "invalid_rule",
                    e.to_string(),
                ));
            }
        }
    }

    Ok(())
}
