mod common;
use common::*;
use proxy_application::ControlPlane;
use proxy_cache::config::Settings;
use proxy_import::{ConflictPolicy, Format};
use std::time::Duration;

#[tokio::test]
#[ignore = "requires a disposable PostgreSQL database; use scripts/postgres_test.py"]
async fn postgres_migrations_crud_import_revision_recovery_and_concurrency() {
    let settings = Settings {
        database_url: std::env::var("TEST_POSTGRES_URL").expect("TEST_POSTGRES_URL"),
        ..Default::default()
    };
    let c = ControlPlane::open(settings.clone()).await.unwrap();
    assert_eq!(c.store.backend.as_str(), "postgresql");
    c.update_draft(|config| {
        config.mappings.push(mapping("pg-one", "/pg-one"));
        Ok(())
    })
    .await
    .unwrap();
    let first = c.publish(0, None).await.unwrap();
    assert_eq!(c.store.draft().await.unwrap().mappings.len(), 1);
    let peer = ControlPlane::open(settings.clone()).await.unwrap();
    let (a, b) = tokio::join!(c.publish(first, None), peer.publish(first, None));
    assert_ne!(
        a.is_ok(),
        b.is_ok(),
        "only one stale concurrent publisher can win"
    );
    let current = c.store.state().await.unwrap().0;
    let document =
        serde_json::to_vec(&serde_json::json!({"mappings":[mapping("pg-import", "/pg-import")]}))
            .unwrap();
    let job = c
        .submit_import(
            "pg.json".into(),
            Format::Json,
            ConflictPolicy::Reject,
            document,
        )
        .await
        .unwrap();
    for _ in 0..1000 {
        let state = c.import_job(&job).await.unwrap();
        if state.status == "ready" {
            break;
        }
        assert!(
            matches!(state.status.as_str(), "pending" | "validating"),
            "{state:?}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(c.import_job(&job).await.unwrap().status, "ready");
    assert_eq!(
        c.store.state().await.unwrap().0,
        current,
        "preview must not publish"
    );
    let committed = c
        .commit_import(&job, current, ConflictPolicy::Reject, true, "pg-commit")
        .await
        .unwrap()
        .revision
        .unwrap();
    assert_eq!(
        c.commit_import(&job, current, ConflictPolicy::Reject, true, "pg-commit")
            .await
            .unwrap()
            .revision
            .unwrap(),
        committed
    );
    assert_eq!(c.store.draft().await.unwrap().mappings.len(), 2);
    for format in [Format::Csv, Format::Xlsx, Format::Json] {
        assert!(
            !proxy_import::export(&c.store.draft().await.unwrap(), format)
                .unwrap()
                .is_empty()
        );
    }
    let restored = c.publish(committed, Some(first)).await.unwrap();
    assert_eq!(c.active.load().mappings.len(), 1);
    drop(peer);
    c.store.pool.close().await;
    drop(c);
    let recovered = ControlPlane::open(settings).await.unwrap();
    assert_eq!(recovered.active.load().revision, restored);
    assert_eq!(recovered.active.load().mappings.len(), 1);
    let preview = recovered.preview_revision_prune(1).await.unwrap();
    assert!(preview.versions.contains(&first));
    let deleted = recovered.prune_revisions(&preview).await.unwrap();
    assert_eq!(deleted, preview.versions.len() as u64);
    assert!(recovered.store.revision(first).await.is_err());
    assert_eq!(recovered.active.load().revision, restored);
    assert!(recovered.store.revision(restored).await.is_ok());
    assert!(recovered.prune_revisions(&preview).await.is_err());
    assert_eq!(
        recovered
            .commit_import(&job, current, ConflictPolicy::Reject, true, "pg-commit")
            .await
            .unwrap()
            .revision,
        Some(committed)
    );
}
