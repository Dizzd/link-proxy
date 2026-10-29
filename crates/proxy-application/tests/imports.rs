mod common;
use common::*;
use proxy_cache::config::ImportLimits;
use proxy_domain::*;
use proxy_import::*;
use std::time::Duration;

async fn ready(control: &proxy_application::ControlPlane, id: &str) -> ImportJob {
    for _ in 0..1000 {
        let job = control.import_job(id).await.unwrap();
        if !matches!(job.status.as_str(), "pending" | "validating") {
            return job;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("job did not finish")
}
fn document(mappings: Vec<Mapping>) -> Vec<u8> {
    serde_json::to_vec(&ImportDocument {
        mappings,
        domains: vec![],
        upstreams: vec![],
    })
    .unwrap()
}
#[test]
fn all_formats_round_trip_nested_rules() {
    let mut m = mapping("a", "/a");
    m.rules.push(serde_json::from_value(serde_json::json!({"name":"Mobile","priority":10,"condition":{"type":"device_type","in":["mobile"]},"action":{"type":"redirect","status":307,"target_url":"https://mobile.example/"}})).unwrap());
    let c = configuration(vec![m]);
    for format in [Format::Csv, Format::Xlsx, Format::Json] {
        let bytes = export(&c, format).unwrap();
        let (parsed, errors) = parse(
            &bytes,
            format,
            &format!("test.{}", format.as_str()),
            &ImportLimits::default(),
            &[],
        )
        .unwrap();
        assert!(errors.is_empty());
        assert_eq!(
            serde_json::to_value(&parsed.mappings).unwrap(),
            serde_json::to_value(&c.mappings).unwrap()
        );
    }
}
#[test]
fn csv_basic_headers_rows_and_encoding() {
    let bytes=b"external_id,mapping_key,source_host,source_path,default_target_url\na,a,example.com,/a,https://target.example/\n";
    assert_eq!(
        parse(bytes, Format::Csv, "a.csv", &ImportLimits::default(), &[])
            .unwrap()
            .0
            .mappings
            .len(),
        1
    );
    assert!(
        parse(
            b"bad,header\nx,y",
            Format::Csv,
            "a.csv",
            &ImportLimits::default(),
            &[]
        )
        .is_err()
    );
    assert!(parse(&[255], Format::Csv, "a.csv", &ImportLimits::default(), &[]).is_err());
}
#[test]
fn file_structure_size_and_json_depth_are_bounded() {
    assert!(
        parse(
            b"not a zip",
            Format::Xlsx,
            "a.xlsx",
            &ImportLimits::default(),
            &[]
        )
        .is_err()
    );
    assert!(parse(b"{}", Format::Json, "a.csv", &ImportLimits::default(), &[]).is_err());
    let limits = ImportLimits {
        max_upload_bytes: 1,
        ..Default::default()
    };
    assert!(parse(b"{}", Format::Json, "a.json", &limits, &[]).is_err());
    assert!(
        parse(
            "[".repeat(60).as_bytes(),
            Format::Json,
            "a.json",
            &ImportLimits::default(),
            &[]
        )
        .is_err()
    );
}
#[test]
fn duplicate_policies_and_complete_rule_replacement() {
    let base = configuration(vec![mapping("a", "/a")]);
    let mut changed = mapping("a", "/changed");
    changed.name = "updated".into();
    let doc = ImportDocument {
        mappings: vec![changed],
        domains: vec![],
        upstreams: vec![],
    };
    assert!(merge(base.clone(), &doc, ConflictPolicy::Reject).is_err());
    assert_eq!(
        merge(base.clone(), &doc, ConflictPolicy::Skip)
            .unwrap()
            .0
            .mappings[0]
            .source_matcher
            .path_value,
        "/a"
    );
    let (merged, result) = merge(base, &doc, ConflictPolicy::Upsert).unwrap();
    assert_eq!(result.updated, 1);
    assert_eq!(merged.mappings[0].name, "updated");
}
#[test]
fn duplicates_within_one_file_are_rejected() {
    let doc = ImportDocument {
        mappings: vec![mapping("a", "/a"), mapping("a", "/a")],
        domains: vec![],
        upstreams: vec![],
    };
    assert!(merge(configuration(vec![]), &doc, ConflictPolicy::Upsert).is_err());
}
#[test]
fn formula_injection_is_escaped() {
    for value in ["=CMD()", "+1", "-1", "@SUM(A1)", "  =CMD()"] {
        assert!(spreadsheet_safe(value).starts_with('\''));
    }
}
#[tokio::test]
async fn dry_run_staging_commit_idempotency_and_rollback() {
    let control = control().await;
    let original = control.active.load_full();
    let id = control
        .submit_import(
            "a.json".into(),
            Format::Json,
            ConflictPolicy::Reject,
            document(vec![mapping("a", "/a")]),
        )
        .await
        .unwrap();
    assert_eq!(ready(&control, &id).await.status, "ready");
    assert!(control.store.draft().await.unwrap().mappings.is_empty());
    assert_eq!(control.active.load().revision, 0);
    assert!(
        control
            .commit_import(&id, 0, ConflictPolicy::Reject, false, "key")
            .await
            .is_err()
    );
    let result = control
        .commit_import(&id, 0, ConflictPolicy::Reject, true, "key")
        .await
        .unwrap();
    assert_eq!(result.revision, Some(1));
    assert!(
        control
            .active
            .load()
            .lookup("example.com", "/a", "GET")
            .is_some()
    );
    assert!(original.lookup("example.com", "/a", "GET").is_none());
    assert_eq!(
        control
            .commit_import(&id, 0, ConflictPolicy::Reject, true, "key")
            .await
            .unwrap()
            .revision,
        Some(1)
    );
    assert!(
        control
            .commit_import(&id, 0, ConflictPolicy::Reject, true, "other")
            .await
            .is_err()
    );
    control
        .update_draft(|c| {
            c.mappings.push(mapping("b", "/b"));
            Ok(())
        })
        .await
        .unwrap();
    control.publish(1, None).await.unwrap();
    control.publish(2, Some(1)).await.unwrap();
    assert_eq!(control.active.load().revision, 3);
    assert!(
        control
            .active
            .load()
            .lookup("example.com", "/b", "GET")
            .is_none()
    );
}
#[tokio::test]
async fn concurrent_draft_change_prevents_staged_import_activation() {
    let c = control().await;
    let id = c
        .submit_import(
            "a.json".into(),
            Format::Json,
            ConflictPolicy::Reject,
            document(vec![mapping("a", "/a")]),
        )
        .await
        .unwrap();
    assert_eq!(ready(&c, &id).await.status, "ready");
    c.update_draft(|c| {
        c.mappings.push(mapping("b", "/b"));
        Ok(())
    })
    .await
    .unwrap();
    assert!(
        c.commit_import(&id, 0, ConflictPolicy::Reject, true, "key")
            .await
            .is_err()
    );
    assert_eq!(c.active.load().revision, 0);
}
#[tokio::test]
async fn cancellation_removes_staging_without_mutation() {
    let c = control().await;
    let id = c
        .submit_import(
            "a.json".into(),
            Format::Json,
            ConflictPolicy::Reject,
            document(vec![mapping("a", "/a")]),
        )
        .await
        .unwrap();
    ready(&c, &id).await;
    c.cancel_import(&id).await.unwrap();
    assert_eq!(c.import_job(&id).await.unwrap().status, "cancelled");
    assert!(c.staged_document(&id).await.unwrap().mappings.is_empty());
    assert_eq!(c.active.load().revision, 0);
}
#[tokio::test]
async fn invalid_destination_and_hostname_never_activate() {
    for (host, target) in [
        ("invalid host", "https://target.example"),
        ("example.com", "file:///etc/passwd"),
    ] {
        let c = control().await;
        let mut m = mapping("a", "/a");
        m.source_matcher.hosts = vec![host.into()];
        if let Action::Redirect { target_url, .. } = &mut m.default_action {
            *target_url = target.into();
        }
        let id = c
            .submit_import(
                "a.json".into(),
                Format::Json,
                ConflictPolicy::Reject,
                document(vec![m]),
            )
            .await
            .unwrap();
        assert_eq!(ready(&c, &id).await.status, "failed");
        assert_eq!(c.active.load().revision, 0);
    }
}
#[tokio::test]
async fn restart_loads_committed_database_revision() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let settings = proxy_cache::config::Settings {
        database_url: format!("sqlite://{}", file.path().display()),
        ..Default::default()
    };
    let c = proxy_application::ControlPlane::open(settings.clone())
        .await
        .unwrap();
    c.update_draft(|c| {
        c.mappings.push(mapping("a", "/a"));
        Ok(())
    })
    .await
    .unwrap();
    c.publish(0, None).await.unwrap();
    sqlx::query("UPDATE config_revisions SET status='committed' WHERE version=1")
        .execute(&c.store.pool)
        .await
        .unwrap();
    c.store.pool.close().await;
    drop(c);
    let recovered = proxy_application::ControlPlane::open(settings)
        .await
        .unwrap();
    assert_eq!(recovered.active.load().revision, 1);
    assert!(
        recovered
            .active
            .load()
            .lookup("example.com", "/a", "GET")
            .is_some()
    );
}
#[tokio::test]
async fn import_one_thousand_records_publishes_once() {
    let c = control().await;
    let maps = (0..1000)
        .map(|i| mapping(&format!("k{i}"), &format!("/k{i}")))
        .collect();
    let id = c
        .submit_import(
            "scale.json".into(),
            Format::Json,
            ConflictPolicy::Reject,
            document(maps),
        )
        .await
        .unwrap();
    assert_eq!(ready(&c, &id).await.status, "ready");
    c.commit_import(&id, 0, ConflictPolicy::Reject, true, "scale")
        .await
        .unwrap();
    assert_eq!(c.active.load().mappings.len(), 1000);
    assert_eq!(c.store.revisions().await.unwrap().len(), 1);
}
#[tokio::test]
#[ignore = "Explicit scale validation: cargo test -p proxy-application --test imports import_large -- --ignored --nocapture"]
async fn import_large_10000_and_50000_records() {
    for count in [10_000, 50_000] {
        let c = control().await;
        let maps = (0..count)
            .map(|i| mapping(&format!("k{i}"), &format!("/k{i}")))
            .collect();
        let start = std::time::Instant::now();
        let id = c
            .submit_import(
                "scale.json".into(),
                Format::Json,
                ConflictPolicy::Reject,
                document(maps),
            )
            .await
            .unwrap();
        assert_eq!(ready(&c, &id).await.status, "ready");
        c.commit_import(&id, 0, ConflictPolicy::Reject, true, "scale")
            .await
            .unwrap();
        println!(
            "{count} records staged and activated in {:?}",
            start.elapsed()
        );
        assert_eq!(c.active.load().mappings.len(), count);
    }
}
