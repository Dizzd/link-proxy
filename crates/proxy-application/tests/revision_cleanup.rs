mod common;
use common::*;
use proxy_application::ControlPlane;
use proxy_cache::config::Settings;
use proxy_storage::Store;

async fn seed(c: &ControlPlane, count: i64) -> i64 {
    let config = c.store.draft().await.unwrap();
    let mut tx = c.store.begin_write().await.unwrap();
    let mut version = Store::state_tx(&mut tx).await.unwrap().0;
    for _ in 0..count {
        version = Store::commit_revision(&mut tx, &config, version, "test_seed")
            .await
            .unwrap();
    }
    sqlx::query("UPDATE config_revisions SET status='superseded' WHERE version<>$1")
        .bind(version)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    c.reconcile().await.unwrap();
    version
}

#[tokio::test]
async fn cleanup_keeps_recent_history_draft_identity_audit_and_import_results() {
    let c = control().await;
    c.update_draft(|config| {
        config.mappings.push(mapping("stable", "/original"));
        Ok(())
    })
    .await
    .unwrap();
    let version = seed(&c, 5).await;
    let draft = c.store.draft().await.unwrap();
    let state = c.store.state().await.unwrap();
    let snapshot = c.active.load_full();
    sqlx::query("INSERT INTO import_jobs(id,file_name,format,conflict_policy,status,base_revision,base_generation,committed_revision,idempotency_key,result) VALUES('old-import','old.json','json','reject','completed',0,0,1,'stable-key',$1)")
        .bind(r#"{"inserted":1,"updated":0,"skipped":0,"revision":1}"#).execute(&c.store.pool).await.unwrap();
    let audit_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_events")
        .fetch_one(&c.store.pool)
        .await
        .unwrap();
    let preview = c.preview_revision_prune(2).await.unwrap();
    assert_eq!(preview.versions, vec![1, 2, 3]);
    assert_eq!(c.prune_revisions(&preview).await.unwrap(), 3);
    assert_eq!(c.store.state().await.unwrap(), state);
    assert!(std::sync::Arc::ptr_eq(&snapshot, &c.active.load_full()));
    assert_eq!(
        serde_json::to_value(c.store.draft().await.unwrap()).unwrap(),
        serde_json::to_value(draft).unwrap()
    );
    assert!(c.store.revision(1).await.is_err());
    assert!(c.store.revision(4).await.is_ok());
    let audit_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_events")
        .fetch_one(&c.store.pool)
        .await
        .unwrap();
    assert_eq!(audit_after, audit_before + 1);
    let retried = c
        .commit_import(
            "old-import",
            0,
            proxy_import::ConflictPolicy::Reject,
            true,
            "stable-key",
        )
        .await
        .unwrap();
    assert_eq!(retried.revision, Some(1));
    assert_eq!(c.publish(version, Some(4)).await.unwrap(), 6);
    assert_eq!(
        c.store.draft().await.unwrap().mappings[0]
            .source_matcher
            .path_value,
        "/original"
    );
}

#[tokio::test]
async fn cleanup_rejects_stale_and_tampered_previews_without_deletion() {
    let c = control().await;
    let version = seed(&c, 4).await;
    let preview = c.preview_revision_prune(1).await.unwrap();
    let mut tampered = preview.clone();
    tampered.versions.push(version);
    assert!(
        c.prune_revisions(&tampered)
            .await
            .unwrap_err()
            .to_string()
            .contains("conflict")
    );
    c.publish(version, None).await.unwrap();
    assert!(
        c.prune_revisions(&preview)
            .await
            .unwrap_err()
            .to_string()
            .contains("conflict")
    );
    assert_eq!(
        c.preview_revision_prune(1).await.unwrap().total_revisions,
        5
    );
    for keep in [0, 10001] {
        assert!(c.preview_revision_prune(keep).await.is_err());
    }
}

#[tokio::test]
async fn cleanup_is_bounded_and_requires_a_fresh_preview_between_batches() {
    let c = control().await;
    seed(&c, 505).await;
    let preview = c.preview_revision_prune(1).await.unwrap();
    assert_eq!(preview.eligible_count, 504);
    assert_eq!(preview.versions.len(), 500);
    assert_eq!(c.prune_revisions(&preview).await.unwrap(), 500);
    assert!(c.prune_revisions(&preview).await.is_err());
    let remaining = c.preview_revision_prune(1).await.unwrap();
    assert_eq!(remaining.versions, vec![501, 502, 503, 504]);
    assert_eq!(c.prune_revisions(&remaining).await.unwrap(), 4);
    assert_eq!(
        c.prune_revisions(&c.preview_revision_prune(1).await.unwrap())
            .await
            .unwrap(),
        0
    );
    assert_eq!(c.publish(505, None).await.unwrap(), 506);
}

#[tokio::test]
async fn cleanup_protects_runtime_database_and_unacknowledged_revisions_then_recovers() {
    let directory = tempfile::tempdir().unwrap();
    let settings = Settings {
        database_url: format!(
            "sqlite://{}",
            directory.path().join("revisions.db").display()
        ),
        ..Default::default()
    };
    let c = ControlPlane::open(settings.clone()).await.unwrap();
    let first = c.publish(0, None).await.unwrap();
    let old_snapshot = c.active.load_full();
    seed(&c, 5).await;
    let config = c.store.draft().await.unwrap();
    let mut tx = c.store.begin_write().await.unwrap();
    let committed = Store::commit_revision(&mut tx, &config, 6, "test_interrupted_commit")
        .await
        .unwrap();
    sqlx::query("UPDATE config_revisions SET status='committed' WHERE version=3")
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    c.active.store(old_snapshot);
    let preview = c.preview_revision_prune(2).await.unwrap();
    assert_eq!(preview.active_revision, first);
    assert_eq!(preview.database_revision, committed);
    assert_eq!(preview.versions, vec![2, 4, 5]);
    c.prune_revisions(&preview).await.unwrap();
    for version in [1, 3, 6, 7] {
        assert!(c.store.revision(version).await.is_ok());
    }
    c.store.pool.close().await;
    drop(c);
    let recovered = ControlPlane::open(settings).await.unwrap();
    assert_eq!(recovered.active.load().revision, committed);
    assert!(recovered.store.revision(first).await.is_ok());
}
