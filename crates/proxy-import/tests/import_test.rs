use proxy_cache::config::ImportLimits;
use proxy_domain::Configuration;
use proxy_import::{Format, export, parse};

#[test]
fn flat_imports_generate_missing_ids_and_preserve_explicit_ids() {
    for format in [Format::Csv, Format::Xlsx] {
        for include_id in [true, false] {
            let mut headers = vec![
                "mapping_key",
                "source_host",
                "source_path",
                "default_target_url",
            ];
            if include_id {
                headers.insert(0, "external_id");
            }
            let rows: Vec<Vec<String>> = (0..3)
                .map(|i| {
                    let mut row = vec![
                        format!("QR{i}"),
                        "example.com".into(),
                        format!("/QR{i}"),
                        "https://target.example/".into(),
                    ];
                    if include_id {
                        row.insert(
                            0,
                            if i == 0 {
                                "stable-id".into()
                            } else {
                                String::new()
                            },
                        );
                    }
                    row
                })
                .collect();
            let bytes = match format {
                Format::Csv => {
                    let mut writer = csv::Writer::from_writer(Vec::new());
                    writer.write_record(&headers).unwrap();
                    for row in &rows {
                        writer.write_record(row).unwrap();
                    }
                    writer.into_inner().unwrap()
                }
                Format::Xlsx => {
                    let mut workbook = rust_xlsxwriter::Workbook::new();
                    let sheet = workbook.add_worksheet();
                    sheet.set_name("Mappings").unwrap();
                    for (column, header) in headers.iter().enumerate() {
                        sheet.write_string(0, column as u16, *header).unwrap();
                    }
                    for (row, values) in rows.iter().enumerate() {
                        for (column, value) in values.iter().enumerate() {
                            if !value.is_empty() {
                                sheet
                                    .write_string((row + 1) as u32, column as u16, value)
                                    .unwrap();
                            }
                        }
                    }
                    workbook.save_to_buffer().unwrap()
                }
                Format::Json => unreachable!(),
            };
            let (document, errors) = parse(
                &bytes,
                format,
                &format!("test.{}", format.as_str()),
                &ImportLimits::default(),
                &[],
            )
            .unwrap();
            assert!(errors.is_empty(), "{errors:?}");
            assert_eq!(document.mappings.len(), 3);
            let ids: std::collections::HashSet<_> =
                document.mappings.iter().map(|m| &m.id).collect();
            assert_eq!(ids.len(), 3);
            assert!(ids.iter().all(|id| !id.is_empty()));
            if include_id {
                assert_eq!(document.mappings[0].id, "stable-id");
            }
            assert_eq!(document.mappings[2].source_matcher.path_value, "/QR2");
        }
    }
}

#[test]
fn empty_external_id_preserves_mapping_json_identity() {
    let config: Configuration = serde_json::from_value(serde_json::json!({
        "mappings": [{"id":"stable-json-id", "mapping_key":"QR1",
            "source_matcher":{"hosts":["example.com"], "path_type":"exact", "path_value":"/QR1"},
            "default_action":{"type":"redirect", "status":302, "target_url":"https://target.example/"}}]
    })).unwrap();
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record(["external_id", "mapping_key", "mapping_json"])
        .unwrap();
    writer
        .write_record([
            "",
            "QR1",
            &serde_json::to_string(&config.mappings[0]).unwrap(),
        ])
        .unwrap();
    let (document, errors) = parse(
        &writer.into_inner().unwrap(),
        Format::Csv,
        "test.csv",
        &ImportLimits::default(),
        &[],
    )
    .unwrap();
    assert!(errors.is_empty(), "{errors:?}");
    assert_eq!(document.mappings[0].id, "stable-json-id");
}
#[test]
fn xlsx_preserves_dependencies_and_negative_rule_priority() {
    let c:Configuration=serde_json::from_value(serde_json::json!({"schema_version":1,"domains":[{"id":"d","hostname":"example.com"}],"upstreams":[{"id":"u","name":"backend","scheme":"http","hostname":"backend.example","port":80}],"mappings":[{"id":"m","mapping_key":"m","source_matcher":{"hosts":["example.com"],"path_type":"exact","path_value":"/a"},"default_action":{"type":"reverse_proxy","upstream_id":"u"},"rules":[{"id":"r","priority":-10,"condition":{"type":"device_type","in":["mobile"]},"action":{"type":"redirect","status":302,"target_url":"https://mobile.example/"}}]}]})).unwrap();
    let bytes = export(&c, Format::Xlsx).unwrap();
    let (parsed, errors) = parse(
        &bytes,
        Format::Xlsx,
        "test.xlsx",
        &ImportLimits::default(),
        &[],
    )
    .unwrap();
    assert!(errors.is_empty());
    assert_eq!(
        serde_json::to_value(parsed.upstreams).unwrap(),
        serde_json::to_value(c.upstreams).unwrap()
    );
    assert_eq!(
        serde_json::to_value(parsed.domains).unwrap(),
        serde_json::to_value(c.domains).unwrap()
    );
    assert_eq!(parsed.mappings[0].rules[0].priority, -10);
}

#[test]
fn semantic_errors_identify_each_invalid_import_record() {
    let data=b"external_id,mapping_key,source_host,source_path,default_target_url\na,a,bad host,/a,https://safe.example/\nb,b,example.com,/b,file:///etc/passwd\n";
    let (_, errors) = parse(
        data,
        Format::Csv,
        "invalid.csv",
        &ImportLimits::default(),
        &[],
    )
    .unwrap();
    assert_eq!(errors.len(), 2);
    assert!(errors[0].location.contains("external_id=a"));
    assert!(errors[1].location.contains("external_id=b"));
}
