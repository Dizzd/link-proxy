#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error("configuration revision conflict")]
    RevisionConflict,
}
