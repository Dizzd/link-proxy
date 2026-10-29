use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceMatcher {
    pub hosts: Vec<String>,
    pub path_type: PathType,
    pub path_value: String,
    #[serde(default)]
    pub methods: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PathType {
    Exact,
    Prefix,
    Template,
}
