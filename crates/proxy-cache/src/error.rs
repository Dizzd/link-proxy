#[derive(Debug, thiserror::Error)]
#[error("cache integrity or revision mismatch")]
pub struct CacheIntegrityError;
