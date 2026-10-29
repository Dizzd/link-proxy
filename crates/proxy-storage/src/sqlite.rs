use sha2::{Digest, Sha256};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatabaseBackend {
    Sqlite,
    Postgres,
}
impl DatabaseBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgresql",
        }
    }
}
#[derive(Clone)]
pub struct Store {
    pub pool: sqlx::AnyPool,
    pub backend: DatabaseBackend,
}
pub fn checksum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
/// SQLx Any QueryBuilder emits question-mark placeholders; number them before execution.
pub fn numbered_parameters(sql: &str) -> String {
    let mut index = 0;
    sql.chars()
        .map(|c| {
            if c == '?' {
                index += 1;
                format!("${index}")
            } else {
                c.to_string()
            }
        })
        .collect()
}
