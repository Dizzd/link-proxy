use crate::router::*;
#[derive(Default, Deserialize)]
pub(crate) struct Page {
    #[serde(default)]
    pub(crate) offset: usize,
    pub(crate) limit: Option<usize>,
    pub(crate) search: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) action: Option<String>,
    pub(crate) enabled: Option<bool>,
    pub(crate) format: Option<Format>,
    pub(crate) ids: Option<String>,
}
impl Page {
    pub(crate) fn limit(&self) -> usize {
        self.limit.unwrap_or(50).clamp(1, 500)
    }
}
#[derive(Deserialize)]
pub(crate) struct Publish {
    pub(crate) expected_revision: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RevisionRetention {
    #[serde(default = "default_revision_retention")]
    pub(crate) keep_latest: u32,
}
fn default_revision_retention() -> u32 {
    100
}
