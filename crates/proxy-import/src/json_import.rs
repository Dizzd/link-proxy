use super::*;
pub(crate) fn parse_format(
    bytes: &[u8],
    _file: &str,
    limits: &ImportLimits,
    _upstreams: &[Upstream],
    document: &mut ImportDocument,
    _errors: &mut Vec<ImportJobError>,
) -> Result<()> {
    check_json_depth(bytes)?;
    *document = serde_json::from_slice(bytes).context("malformed JSON import document")?;
    for (i, m) in document.mappings.iter().enumerate() {
        ensure!(
            serde_json::to_vec(m)?.len() <= limits.max_record_bytes,
            "JSON mapping /mappings/{i} exceeds record limit"
        );
    }

    Ok(())
}
