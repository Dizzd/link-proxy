use crate::Store;
use anyhow::{Result, ensure};
#[derive(Clone, Debug)]
pub struct AdminUser {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub enabled: bool,
}
#[derive(Clone, Debug)]
pub struct AdminSession {
    pub user_id: String,
    pub username: String,
    pub expires_at: i64,
}
impl Store {
    pub async fn admin_count(&self) -> Result<i64> {
        Ok(sqlx::query_scalar("SELECT count(*) FROM admin_users")
            .fetch_one(&self.pool)
            .await?)
    }
    pub async fn admin_user(&self, username: &str) -> Result<Option<AdminUser>> {
        let row: Option<(String, String, String, i64)> = sqlx::query_as(
            "SELECT id,username,password_hash,enabled FROM admin_users WHERE username=$1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(id, username, password_hash, enabled)| AdminUser {
            id,
            username,
            password_hash,
            enabled: enabled == 1,
        }))
    }
    pub async fn bootstrap_admin(
        &self,
        id: &str,
        username: &str,
        password_hash: &str,
    ) -> Result<bool> {
        let mut tx = self.begin_write().await?;
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM admin_users")
            .fetch_one(&mut *tx)
            .await?;
        if count != 0 {
            return Ok(false);
        }
        let now = chrono::Utc::now().timestamp();
        sqlx::query("INSERT INTO admin_users(id,username,password_hash,enabled,created_at,updated_at) VALUES($1,$2,$3,1,$4,$5)").bind(id).bind(username).bind(password_hash).bind(now).bind(now).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO audit_events(operation,revision,actor,resource_id) SELECT 'auth.bootstrap',active_revision,$1,$2 FROM control_state WHERE id=1").bind(username).bind(id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn create_session(
        &self,
        user: &AdminUser,
        token_hash: &str,
        expires_at: i64,
    ) -> Result<bool> {
        let mut tx = self.begin_write().await?;
        let current: Option<(String, i64)> =
            sqlx::query_as("SELECT password_hash,enabled FROM admin_users WHERE id=$1")
                .bind(&user.id)
                .fetch_optional(&mut *tx)
                .await?;
        if current
            .as_ref()
            .is_none_or(|(hash, enabled)| hash != &user.password_hash || *enabled != 1)
        {
            return Ok(false);
        }
        sqlx::query("DELETE FROM admin_sessions WHERE expires_at <= $1")
            .bind(chrono::Utc::now().timestamp())
            .execute(&mut *tx)
            .await?;
        // Keep nine previous sessions before adding this login, for a maximum of ten.
        sqlx::query("DELETE FROM admin_sessions WHERE user_id=$1 AND id NOT IN (SELECT id FROM admin_sessions WHERE user_id=$2 ORDER BY created_at DESC,id DESC LIMIT 9)").bind(&user.id).bind(&user.id).execute(&mut *tx).await?;
        sqlx::query(
            "INSERT INTO admin_sessions(id,user_id,created_at,expires_at) VALUES($1,$2,$3,$4)",
        )
        .bind(token_hash)
        .bind(&user.id)
        .bind(chrono::Utc::now().timestamp())
        .bind(expires_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO audit_events(operation,revision,actor,resource_id) SELECT 'auth.login',active_revision,$1,$2 FROM control_state WHERE id=1").bind(&user.username).bind(&user.id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(true)
    }
    pub async fn session(&self, token_hash: &str) -> Result<Option<AdminSession>> {
        let row:Option<(String,String,i64)>=sqlx::query_as("SELECT s.user_id,u.username,s.expires_at FROM admin_sessions s JOIN admin_users u ON u.id=s.user_id WHERE s.id=$1 AND s.expires_at>$2 AND u.enabled=1").bind(token_hash).bind(chrono::Utc::now().timestamp()).fetch_optional(&self.pool).await?;
        Ok(row.map(|(user_id, username, expires_at)| AdminSession {
            user_id,
            username,
            expires_at,
        }))
    }
    pub async fn revoke_session(&self, token_hash: &str, session: &AdminSession) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM admin_sessions WHERE id=$1")
            .bind(token_hash)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO audit_events(operation,revision,actor,resource_id) SELECT 'auth.logout',active_revision,$1,$2 FROM control_state WHERE id=1").bind(&session.username).bind(&session.user_id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn change_admin_password(&self, user: &AdminUser, new_hash: &str) -> Result<()> {
        let mut tx = self.begin_write().await?;
        let result=sqlx::query("UPDATE admin_users SET password_hash=$1,updated_at=$2 WHERE id=$3 AND password_hash=$4 AND enabled=1").bind(new_hash).bind(chrono::Utc::now().timestamp()).bind(&user.id).bind(&user.password_hash).execute(&mut *tx).await?;
        ensure!(
            result.rows_affected() == 1,
            "administrator credentials changed; sign in again"
        );
        sqlx::query("DELETE FROM admin_sessions WHERE user_id=$1")
            .bind(&user.id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO audit_events(operation,revision,actor,resource_id) SELECT 'auth.password_change',active_revision,$1,$2 FROM control_state WHERE id=1").bind(&user.username).bind(&user.id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn audit_auth_failure(&self, username: &str) -> Result<()> {
        sqlx::query("INSERT INTO audit_events(operation,revision,actor) SELECT 'auth.login_failed',active_revision,$1 FROM control_state WHERE id=1").bind(username).execute(&self.pool).await?;
        Ok(())
    }
}
