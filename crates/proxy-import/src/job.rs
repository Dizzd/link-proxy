use super::*;
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    Reject,
    Skip,
    Upsert,
}
impl ConflictPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Skip => "skip",
            Self::Upsert => "upsert",
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Csv,
    Xlsx,
    Json,
}
impl Format {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Xlsx => "xlsx",
            Self::Json => "json",
        }
    }
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Csv => "text/csv; charset=utf-8",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Json => "application/json",
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportJob {
    pub id: String,
    pub file_name: String,
    pub format: String,
    pub conflict_policy: String,
    pub status: String,
    pub base_revision: i64,
    pub base_generation: i64,
    pub total_records: i64,
    pub valid_records: i64,
    pub error_count: i64,
    pub result: Option<String>,
    pub committed_revision: Option<i64>,
    pub created_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportJobError {
    pub file_name: String,
    pub location: String,
    pub field: String,
    pub code: String,
    pub description: String,
    pub suggestion: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ImportJobResult {
    pub inserted: usize,
    pub updated: usize,
    pub skipped: usize,
    pub revision: Option<i64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImportDocument {
    #[serde(default)]
    pub domains: Vec<Domain>,
    #[serde(default)]
    pub upstreams: Vec<Upstream>,
    pub mappings: Vec<Mapping>,
}
