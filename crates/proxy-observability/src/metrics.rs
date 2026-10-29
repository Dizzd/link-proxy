use prometheus::{Encoder, Histogram, IntCounter, IntGauge, Registry, TextEncoder};

pub struct Metrics {
    registry: Registry,
    pub requests: IntCounter,
    pub request_duration: Histogram,
    pub lookup_duration: Histogram,
    pub active_mappings: IntGauge,
    pub revision: IntGauge,
    pub compile_duration: Histogram,
    pub import_jobs: IntCounter,
    pub import_records: IntCounter,
    pub import_errors: IntCounter,
    pub import_duration: Histogram,
    pub redis_connected: IntGauge,
    pub redis_errors: IntCounter,
    pub revision_lag: IntGauge,
    pub sync_duration: Histogram,
}
impl Metrics {
    pub fn new() -> anyhow::Result<Self> {
        let registry = Registry::new();
        macro_rules! metric {
            (Histogram, $name:literal, $help:literal) => {{
                let metric = Histogram::with_opts(prometheus::HistogramOpts::new($name, $help))?;
                registry.register(Box::new(metric.clone()))?;
                metric
            }};
            ($kind:ident, $name:literal, $help:literal) => {{
                let metric = $kind::new($name, $help)?;
                registry.register(Box::new(metric.clone()))?;
                metric
            }};
        }
        Ok(Self {
            requests: metric!(IntCounter, "proxy_requests_total", "Total proxy requests"),
            request_duration: metric!(
                Histogram,
                "proxy_request_duration_seconds",
                "Request latency"
            ),
            lookup_duration: metric!(
                Histogram,
                "proxy_route_lookup_duration_seconds",
                "Route lookup latency"
            ),
            active_mappings: metric!(
                IntGauge,
                "proxy_active_mappings",
                "Enabled mappings in local snapshot"
            ),
            revision: metric!(IntGauge, "proxy_config_revision", "Local active revision"),
            compile_duration: metric!(
                Histogram,
                "proxy_config_compile_duration_seconds",
                "Snapshot compile latency"
            ),
            import_jobs: metric!(
                IntCounter,
                "proxy_import_jobs_total",
                "Import jobs accepted"
            ),
            import_records: metric!(
                IntCounter,
                "proxy_import_records_processed_total",
                "Parsed import records"
            ),
            import_errors: metric!(
                IntCounter,
                "proxy_import_validation_errors_total",
                "Import validation errors"
            ),
            import_duration: metric!(
                Histogram,
                "proxy_import_duration_seconds",
                "Import validation latency"
            ),
            redis_connected: metric!(
                IntGauge,
                "proxy_redis_connection_status",
                "Redis last operation success"
            ),
            redis_errors: metric!(
                IntCounter,
                "proxy_redis_cache_errors_total",
                "Redis failures"
            ),
            revision_lag: metric!(
                IntGauge,
                "proxy_cache_revision_lag",
                "Database revision minus local revision"
            ),
            sync_duration: metric!(
                Histogram,
                "proxy_cache_sync_duration_seconds",
                "Reconciliation latency"
            ),
            registry,
        })
    }
    pub fn render(&self) -> anyhow::Result<String> {
        let mut bytes = Vec::new();
        TextEncoder::new().encode(&self.registry.gather(), &mut bytes)?;
        Ok(String::from_utf8(bytes)?)
    }
}
