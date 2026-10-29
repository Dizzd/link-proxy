fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all("tests/fixtures")?;
    std::fs::create_dir_all("examples/import")?;
    let config: proxy_domain::Configuration = serde_json::from_value(
        serde_json::json!({"schema_version":1,"domains":[{"id":"domain-abc","hostname":"abc.example.com"}],"upstreams":[{"id":"backend-01","name":"backend","scheme":"http","hostname":"backend.example.com","port":8080}],"mappings":[{"id":"advanced-001","mapping_key":"QRADV","name":"Conditional campaign","source_matcher":{"hosts":["abc.example.com"],"path_type":"exact","path_value":"/QRADV","methods":["GET"]},"rules":[{"id":"weekday-mobile","name":"Weekday mobile","priority":20,"condition":{"type":"all","conditions":[{"type":"device_type","in":["mobile"]},{"type":"time_window","timezone":"Asia/Ho_Chi_Minh","days_of_week":["mon","tue","wed","thu","fri"],"start":"08:00","end":"18:00"}]},"action":{"type":"redirect","status":302,"target_url":"https://mobile.example.com/"}},{"id":"weekend","name":"Weekend","priority":10,"condition":{"type":"time_window","timezone":"Asia/Ho_Chi_Minh","days_of_week":["sat","sun"],"start":"00:00","end":"23:59"},"action":{"type":"redirect","status":302,"target_url":"https://weekend.example.com/"}}],"default_action":{"type":"redirect","status":302,"target_url":"https://default.example.com/"}},{"id":"advanced-proxy","mapping_key":"api","name":"Template API proxy","source_matcher":{"hosts":["abc.example.com","second.example.com"],"path_type":"template","path_value":"/api/{id}","methods":["GET","POST"]},"default_action":{"type":"reverse_proxy","upstream_id":"backend-01","rewrite":"/v1/{id}","preserve_host":false}}]}),
    )?;
    for format in [
        proxy_import::Format::Csv,
        proxy_import::Format::Xlsx,
        proxy_import::Format::Json,
    ] {
        let basic = proxy_import::template(format)?;
        std::fs::write(
            format!("tests/fixtures/mappings.{}", format.as_str()),
            &basic,
        )?;
        let bytes = if matches!(format, proxy_import::Format::Csv) {
            basic
        } else {
            proxy_import::export(&config, format)?
        };
        std::fs::write(
            format!(
                "examples/import/{}.{}",
                if matches!(format, proxy_import::Format::Csv) {
                    "basic"
                } else {
                    "advanced"
                },
                format.as_str()
            ),
            bytes,
        )?;
    }
    std::fs::write(
        "tests/fixtures/conditional_mappings.json",
        proxy_import::export(&config, proxy_import::Format::Json)?,
    )?;
    std::fs::write("tests/fixtures/invalid_mappings.csv",b"external_id,mapping_key,source_host,source_path,default_target_url\nbad,bad,invalid host,/bad,file:///etc/passwd\n")?;
    Ok(())
}
