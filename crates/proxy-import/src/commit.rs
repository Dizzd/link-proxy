use super::*;
pub fn merge(
    mut base: Configuration,
    document: &ImportDocument,
    policy: ConflictPolicy,
) -> Result<(Configuration, ImportJobResult)> {
    let mut summary = ImportJobResult::default();
    let mut domains: HashMap<String, usize> = base
        .domains
        .iter()
        .enumerate()
        .map(|(i, d)| (d.id.clone(), i))
        .collect();
    for domain in &document.domains {
        if let Some(i) = domains.get(&domain.id) {
            match policy {
                ConflictPolicy::Reject => bail!("domain identity conflict: {}", domain.id),
                ConflictPolicy::Skip => {}
                ConflictPolicy::Upsert => base.domains[*i] = domain.clone(),
            }
        } else {
            domains.insert(domain.id.clone(), base.domains.len());
            base.domains.push(domain.clone());
        }
    }
    let mut upstreams: HashMap<String, usize> = base
        .upstreams
        .iter()
        .enumerate()
        .map(|(i, u)| (u.id.clone(), i))
        .collect();
    for u in &document.upstreams {
        if let Some(i) = upstreams.get(&u.id) {
            match policy {
                ConflictPolicy::Reject => bail!("upstream identity conflict: {}", u.id),
                ConflictPolicy::Skip => {}
                ConflictPolicy::Upsert => base.upstreams[*i] = u.clone(),
            }
        } else {
            upstreams.insert(u.id.clone(), base.upstreams.len());
            base.upstreams.push(u.clone());
        }
    }
    let mut index: HashMap<String, usize> = base
        .mappings
        .iter()
        .enumerate()
        .map(|(i, m)| (m.id.clone(), i))
        .collect();
    let mut keys: HashMap<String, String> = base
        .mappings
        .iter()
        .map(|m| (m.mapping_key.clone(), m.id.clone()))
        .collect();
    let mut seen = HashSet::new();
    let mut seen_keys = HashSet::new();
    for m in &document.mappings {
        ensure!(
            seen.insert(&m.id) && seen_keys.insert(&m.mapping_key),
            "duplicate identity or key within import: {}",
            m.mapping_key
        );
        if let Some(existing_id) = keys.get(&m.mapping_key) {
            ensure!(
                *existing_id == m.id,
                "mapping key belongs to a different external_id: {}",
                m.mapping_key
            );
        }
        if let Some(i) = index.get(&m.id) {
            match policy {
                ConflictPolicy::Reject => bail!("existing mapping identity: {}", m.id),
                ConflictPolicy::Skip => summary.skipped += 1,
                ConflictPolicy::Upsert => {
                    keys.remove(&base.mappings[*i].mapping_key);
                    keys.insert(m.mapping_key.clone(), m.id.clone());
                    let mut mapping = m.clone();
                    mapping.created_at = base.mappings[*i].created_at;
                    mapping.updated_at = now();
                    base.mappings[*i] = mapping;
                    summary.updated += 1;
                }
            }
        } else {
            index.insert(m.id.clone(), base.mappings.len());
            keys.insert(m.mapping_key.clone(), m.id.clone());
            base.mappings.push(m.clone());
            summary.inserted += 1;
        }
    }
    Ok((base, summary))
}
