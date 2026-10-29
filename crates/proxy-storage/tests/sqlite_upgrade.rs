use proxy_domain::Configuration;
use proxy_storage::{DatabaseBackend, Store};

#[tokio::test]
async fn additive_auth_migration_preserves_existing_sqlite_configuration() {
    let directory = tempfile::tempdir().unwrap();
    let legacy = directory.path().join("legacy-migrations");
    std::fs::create_dir(&legacy).unwrap();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations/sqlite");
    for name in [
        "0001_initial.sql",
        "0002_import_jobs.sql",
        "0003_indexes.sql",
    ] {
        std::fs::copy(source.join(name), legacy.join(name)).unwrap();
    }
    let url = format!(
        "sqlite://{}?mode=rwc",
        directory.path().join("existing.db").display()
    );
    sqlx::any::install_default_drivers();
    let pool = sqlx::AnyPool::connect(&url).await.unwrap();
    sqlx::migrate::Migrator::new(legacy.as_path())
        .await
        .unwrap()
        .run(&pool)
        .await
        .unwrap();
    let store = Store {
        pool,
        backend: DatabaseBackend::Sqlite,
    };
    let config: Configuration = serde_json::from_value(serde_json::json!({"schema_version":1,"mappings":[{
        "id":"existing-id","mapping_key":"existing-key",
        "source_matcher":{"hosts":["existing.example"],"path_type":"exact","path_value":"/keep-this-url"},
        "default_action":{"type":"redirect","status":302,"target_url":"https://original.example/"}
    }]})).unwrap();
    let mut tx = store.begin_write().await.unwrap();
    Store::write_draft(&mut tx, &config).await.unwrap();
    let revision = Store::commit_revision(&mut tx, &config, 0, "legacy-test")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    store.pool.close().await;
    let upgraded = Store::connect(&url).await.unwrap();
    let (active, recovered) = upgraded.committed().await.unwrap();
    assert_eq!(active, revision);
    assert_eq!(recovered.mappings[0].id, "existing-id");
    assert_eq!(
        recovered.mappings[0].source_matcher.path_value,
        "/keep-this-url"
    );
    assert_eq!(
        serde_json::to_value(&recovered).unwrap(),
        serde_json::to_value(&config).unwrap()
    );
    assert_eq!(upgraded.admin_count().await.unwrap(), 0);
    assert_eq!(upgraded.revisions().await.unwrap().len(), 1);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success=1")
        .fetch_one(&upgraded.pool)
        .await
        .unwrap();
    assert_eq!(count, 4);
}
