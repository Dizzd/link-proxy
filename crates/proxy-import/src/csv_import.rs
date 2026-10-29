use super::*;
pub(crate) fn parse_format(
    bytes: &[u8],
    file: &str,
    limits: &ImportLimits,
    upstreams: &[Upstream],
    document: &mut ImportDocument,
    errors: &mut Vec<ImportJobError>,
) -> Result<()> {
    std::str::from_utf8(bytes).context("CSV must be UTF-8")?;
    let mut reader = csv::ReaderBuilder::new().from_reader(bytes);
    let headers = reader.headers()?.clone();
    for required in ["external_id", "mapping_key"] {
        ensure!(
            headers.iter().any(|h| h == required),
            "missing header: {required}"
        );
    }
    for (i, row) in reader.records().enumerate() {
        ensure!(
            i < limits.max_records,
            "record count exceeds configured limit"
        );
        let parsed: Result<Mapping> = (|| {
            let row = row?;
            ensure!(
                row.iter().map(str::len).sum::<usize>() <= limits.max_record_bytes,
                "CSV record too large"
            );
            let flat: FlatMapping = row.deserialize(Some(&headers))?;
            flat.mapping(upstreams)
        })();
        match parsed {
            Ok(m) => document.mappings.push(m),
            Err(e) => errors.push(error(
                file,
                format!("row {}", i + 2),
                "invalid_record",
                e.to_string(),
            )),
        }
    }

    Ok(())
}
