pub fn server_configuration() -> pingora::server::configuration::ServerConf {
    pingora::server::configuration::ServerConf {
        grace_period_seconds: Some(2),
        graceful_shutdown_timeout_seconds: Some(10),
        ..Default::default()
    }
}
