use super::*;
pub(crate) fn check_json_depth(bytes: &[u8]) -> Result<()> {
    let (mut depth, mut quoted, mut escape) = (0_usize, false, false);
    for b in bytes {
        if quoted {
            if escape {
                escape = false;
            } else if *b == b'\\' {
                escape = true;
            } else if *b == b'"' {
                quoted = false;
            }
        } else {
            match b {
                b'"' => quoted = true,
                b'{' | b'[' => {
                    depth += 1;
                    ensure!(depth <= 48, "JSON nesting exceeds 48 levels");
                }
                b'}' | b']' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    Ok(())
}

/// Validate individual records without constructing a routing snapshot per row.
pub(crate) fn validate_record(
    mapping: &Mapping,
    upstreams: &HashMap<String, Upstream>,
) -> Result<()> {
    ensure!(
        !mapping.mapping_key.is_empty()
            && mapping.mapping_key.len() <= 128
            && mapping
                .mapping_key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)),
        "invalid mapping_key"
    );
    proxy_routing::validate_path(&mapping.source_matcher.path_value)?;
    ensure!(
        !mapping.source_matcher.hosts.is_empty(),
        "source hosts required"
    );
    for host in &mapping.source_matcher.hosts {
        proxy_routing::normalize_host(host)?;
    }
    proxy_routing::validate_action(&mapping.default_action, upstreams, &[])?;
    ensure!(mapping.rules.len() <= 64, "mapping exceeds 64 rules");
    for rule in &mapping.rules {
        proxy_routing::conditions::CompiledCondition::compile(&rule.condition)?;
        proxy_routing::validate_action(&rule.action, upstreams, &[])?;
    }
    Ok(())
}
