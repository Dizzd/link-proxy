use anyhow::Result;
use proxy_domain::*;
use sqlx::Transaction;

use crate::Store;
impl Store {
    pub async fn draft(&self) -> Result<Configuration> {
        let mut tx = self.pool.begin().await?;
        let config = Self::draft_tx(&mut tx).await?;
        tx.commit().await?;
        Ok(config)
    }
    pub async fn draft_tx(tx: &mut Transaction<'_, sqlx::Any>) -> Result<Configuration> {
        let domains: Vec<String> = sqlx::query_scalar("SELECT data FROM domains ORDER BY id")
            .fetch_all(&mut **tx)
            .await?;
        let upstreams: Vec<String> = sqlx::query_scalar("SELECT data FROM upstreams ORDER BY id")
            .fetch_all(&mut **tx)
            .await?;
        let mappings: Vec<String> = sqlx::query_scalar("SELECT data FROM mappings ORDER BY id")
            .fetch_all(&mut **tx)
            .await?;
        Ok(Configuration {
            schema_version: 1,
            domains: domains
                .iter()
                .map(|s| serde_json::from_str(s))
                .collect::<Result<_, _>>()?,
            upstreams: upstreams
                .iter()
                .map(|s| serde_json::from_str(s))
                .collect::<Result<_, _>>()?,
            mappings: mappings
                .iter()
                .map(|s| serde_json::from_str(s))
                .collect::<Result<_, _>>()?,
        })
    }
    pub async fn write_draft(
        tx: &mut Transaction<'_, sqlx::Any>,
        config: &Configuration,
    ) -> Result<()> {
        // Replace only generic draft tables inside one transaction. Immutable revisions and legacy tables are untouched.
        sqlx::query("DELETE FROM mappings")
            .execute(&mut **tx)
            .await?;
        sqlx::query("DELETE FROM domains")
            .execute(&mut **tx)
            .await?;
        sqlx::query("DELETE FROM upstreams")
            .execute(&mut **tx)
            .await?;
        for d in &config.domains {
            sqlx::query("INSERT INTO domains(id,hostname,enabled,data) VALUES($1,$2,$3,$4)")
                .bind(&d.id)
                .bind(&d.hostname)
                .bind(i64::from(d.enabled))
                .bind(serde_json::to_string(d)?)
                .execute(&mut **tx)
                .await?;
        }
        for u in &config.upstreams {
            sqlx::query("INSERT INTO upstreams(id,name,data) VALUES($1,$2,$3)")
                .bind(&u.id)
                .bind(&u.name)
                .bind(serde_json::to_string(u)?)
                .execute(&mut **tx)
                .await?;
        }
        for m in &config.mappings {
            sqlx::query(
                "INSERT INTO mappings(id,mapping_key,enabled,priority,data) VALUES($1,$2,$3,$4,$5)",
            )
            .bind(&m.id)
            .bind(&m.mapping_key)
            .bind(i64::from(m.enabled))
            .bind(m.priority)
            .bind(serde_json::to_string(m)?)
            .execute(&mut **tx)
            .await?;
            for h in &m.source_matcher.hosts {
                sqlx::query("INSERT INTO mapping_hosts(mapping_id,hostname) VALUES($1,$2)")
                    .bind(&m.id)
                    .bind(h.to_ascii_lowercase())
                    .execute(&mut **tx)
                    .await?;
            }
            for r in &m.rules {
                sqlx::query("INSERT INTO mapping_rules(id,mapping_id,priority,enabled,condition_schema_version,data) VALUES($1,$2,$3,$4,$5,$6)").bind(&r.id).bind(&m.id).bind(r.priority).bind(i64::from(r.enabled)).bind(i64::from(r.condition_schema_version)).bind(serde_json::to_string(r)?).execute(&mut **tx).await?;
            }
        }
        sqlx::query("UPDATE control_state SET draft_generation=draft_generation+1 WHERE id=1")
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}
