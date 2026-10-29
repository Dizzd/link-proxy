pub fn load() -> anyhow::Result<proxy_cache::config::Settings> {
    let path = std::env::var("PROXY_CONFIG").unwrap_or_else(|_| "config.toml".into());
    proxy_cache::config::Settings::load(std::path::Path::new(&path))
}
