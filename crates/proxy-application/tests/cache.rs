mod common;
use common::*;
use proxy_application::ControlPlane;
use proxy_cache::{config::*, *};

#[tokio::test]
async fn memory_mode_ignores_enabled_redis_and_invalid_environment() {
    let mut settings = Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    };
    settings.cache.redis.enabled = true;
    settings.cache.redis.url_env = "NONEXISTENT_PROXY_TEST_URL".into();
    let c = ControlPlane::open(settings).await.unwrap();
    c.reconcile().await.unwrap();
    assert!(c.cache.status.read().await.connected);
    assert_eq!(c.cache.status.read().await.mode, CacheMode::Memory);
}
#[test]
fn cache_envelope_rejects_stale_revision_and_tampering() {
    let mut envelope =
        CachedConfiguration::new(2, &configuration(vec![mapping("a", "/a")])).unwrap();
    assert!(envelope.validate(1, &envelope.checksum).is_err());
    envelope.data.push(' ');
    assert!(envelope.validate(2, &envelope.checksum).is_err());
}
#[tokio::test]
async fn memory_cache_is_distinct_from_active_snapshot() {
    let cache = MemoryConfigurationCache::default();
    let config = configuration(vec![mapping("a", "/a")]);
    cache
        .save(CachedConfiguration::new(1, &config).unwrap())
        .await
        .unwrap();
    assert!(cache.load(2).await.unwrap().is_none());
    assert!(cache.load(1).await.unwrap().is_some());
}
#[cfg(feature = "redis-cache")]
#[tokio::test]
async fn redis_unavailable_does_not_prevent_publish_or_requests() {
    let mut settings = Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    };
    settings.cache.mode = CacheMode::MemoryRedis;
    settings.cache.redis.enabled = true;
    settings.cache.redis.url_env = "MISSING_PROXY_REDIS_TEST".into();
    let c = ControlPlane::open(settings).await.unwrap();
    c.reconcile().await.unwrap();
    assert!(!c.cache.status.read().await.connected);
    c.update_draft(|c| {
        c.mappings.push(mapping("a", "/a"));
        Ok(())
    })
    .await
    .unwrap();
    c.publish(0, None).await.unwrap();
    for _ in 0..1000 {
        assert!(c.active.load().lookup("example.com", "/a", "GET").is_some());
    }
    assert_eq!(c.active.load().revision, 1);
}
#[cfg(feature = "redis-cache")]
#[tokio::test]
#[ignore = "Requires REDIS_TEST_URL and a running real Redis server"]
async fn real_redis_versioned_cache_and_reconciliation() {
    assert!(std::env::var("REDIS_TEST_URL").is_ok());
    let mut settings = Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    };
    settings.cache.mode = CacheMode::MemoryRedis;
    settings.cache.redis.enabled = true;
    settings.cache.redis.url_env = "REDIS_TEST_URL".into();
    settings.cache.redis.key_prefix = format!("test:{}:", uuid::Uuid::new_v4());
    let c = ControlPlane::open(settings).await.unwrap();
    c.update_draft(|c| {
        c.mappings.push(mapping("a", "/a"));
        Ok(())
    })
    .await
    .unwrap();
    c.publish(0, None).await.unwrap();
    c.reconcile().await.unwrap();
    assert!(c.cache.status.read().await.connected);
}

#[cfg(feature = "redis-cache")]
#[tokio::test]
#[ignore = "Requires disposable Redis: python3 scripts/redis_test.py"]
async fn real_redis_notifications_are_advisory_and_newer_cache_is_not_activated() {
    let mut settings = Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    };
    settings.cache.mode = CacheMode::MemoryRedis;
    settings.cache.redis.enabled = true;
    settings.cache.redis.url_env = "REDIS_TEST_URL".into();
    settings.cache.redis.key_prefix = format!("test:{}:", uuid::Uuid::new_v4());
    let c = ControlPlane::open(settings.clone()).await.unwrap();
    let mut hints = CacheService::notifications(&settings.cache).unwrap();
    let mut observed = false;
    for revision in 1..40 {
        c.cache.synchronize(revision, &configuration(vec![])).await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        if hints.try_recv().is_ok() {
            observed = true;
            break;
        }
    }
    assert!(
        observed,
        "Pub/Sub listener did not receive a revision notification"
    );
    c.reconcile().await.unwrap();
    assert_eq!(c.active.load().revision, 0);
}
#[cfg(feature = "redis-cache")]
#[tokio::test]
#[ignore = "Shuts down disposable Redis; run only through scripts/redis_test.py"]
async fn real_redis_z_failure_retains_live_snapshot() {
    let mut settings = Settings {
        database_url: "sqlite::memory:".into(),
        ..Default::default()
    };
    settings.cache.mode = CacheMode::MemoryRedis;
    settings.cache.redis.enabled = true;
    settings.cache.redis.url_env = "REDIS_TEST_URL".into();
    settings.cache.redis.key_prefix = format!("test:{}:", uuid::Uuid::new_v4());
    let c = ControlPlane::open(settings).await.unwrap();
    c.update_draft(|config| {
        config.mappings.push(mapping("redis-loss", "/redis-loss"));
        Ok(())
    })
    .await
    .unwrap();
    c.publish(0, None).await.unwrap();
    c.reconcile().await.unwrap();
    assert!(c.cache.status.read().await.connected);
    let client = ::redis::Client::open(std::env::var("REDIS_TEST_URL").unwrap()).unwrap();
    let mut connection = client.get_multiplexed_async_connection().await.unwrap();
    let _: Result<String, _> = ::redis::cmd("SHUTDOWN")
        .arg("NOSAVE")
        .query_async(&mut connection)
        .await;
    c.reconcile().await.unwrap();
    assert!(!c.cache.status.read().await.connected);
    for _ in 0..1000 {
        assert!(
            c.active
                .load()
                .lookup("example.com", "/redis-loss", "GET")
                .is_some()
        );
    }
    assert_eq!(c.active.load().revision, 1);
}
