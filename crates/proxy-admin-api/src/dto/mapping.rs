use crate::router::*;
#[derive(Deserialize)]
pub(crate) struct Simulation {
    pub(crate) host: String,
    pub(crate) path: String,
    #[serde(default = "get_method")]
    pub(crate) method: String,
    #[serde(default)]
    pub(crate) user_agent: String,
    #[serde(default)]
    pub(crate) query: String,
    pub(crate) now: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub(crate) draft: bool,
}
fn get_method() -> String {
    "GET".into()
}
