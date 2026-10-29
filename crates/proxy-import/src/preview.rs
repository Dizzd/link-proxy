use serde::Serialize;
#[derive(Serialize)]
pub struct ImportPreview {
    pub job: crate::ImportJob,
    pub items: Vec<crate::StagedRecord>,
}
