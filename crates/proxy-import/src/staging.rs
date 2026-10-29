use super::*;
#[derive(Serialize, Deserialize)]
pub enum StagedRecord {
    Mapping(Mapping),
    Domain(Domain),
    Upstream(Upstream),
}
