use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Redirect {
        status: u16,
        target_url: String,
        #[serde(default)]
        path_handling: PathHandling,
        #[serde(default)]
        query_policy: QueryPolicy,
    },
    ReverseProxy {
        upstream_id: String,
        #[serde(default)]
        rewrite: Option<String>,
        #[serde(default)]
        preserve_host: bool,
    },
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathHandling {
    #[default]
    Replace,
    Append,
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryPolicy {
    #[default]
    Drop,
    Preserve,
    Merge,
}
