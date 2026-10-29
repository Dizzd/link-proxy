use crate::router::*;
#[derive(Deserialize)]
pub(crate) struct CommitImport {
    pub(crate) import_job_id: String,
    pub(crate) expected_revision: i64,
    pub(crate) conflict_policy: ConflictPolicy,
    pub(crate) confirmation: bool,
}
