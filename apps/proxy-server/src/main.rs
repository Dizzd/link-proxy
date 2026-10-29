use proxy_application::ControlPlane;

mod bootstrap;
mod config;
mod runtime;
mod shutdown;

fn main() -> anyhow::Result<()> {
    proxy_observability::init_logging();
    let settings = config::load()?;
    let token = if settings.auth.api_token_enabled {
        std::env::var(&settings.admin_token_env)
            .map_err(|_| anyhow::anyhow!("API token environment variable is missing"))?
    } else {
        String::new()
    };
    let runtime = tokio::runtime::Runtime::new()?;
    let control = runtime.block_on(ControlPlane::open(settings.clone()))?;
    let reset = std::env::args().any(|arg| arg == "--reset-admin-password");
    runtime.block_on(proxy_admin_api::bootstrap_admin(&control, reset))?;
    if reset {
        return Ok(());
    }
    let app = bootstrap::admin_router(control.clone(), &token)?;
    let admin = runtime.block_on(tokio::net::TcpListener::bind(&settings.admin_listen))?;
    tracing::info!(admin=%settings.admin_listen,proxy=%settings.proxy_listen,"Dynamic URL Mapping & Conditional Routing Engine starting");
    runtime.spawn(async move {
        if let Err(error) = axum::serve(admin, app).await {
            tracing::error!(%error,"admin server stopped");
        }
    });
    runtime.spawn(runtime::reconciliation(control.clone()));
    {
        let conf = shutdown::server_configuration();
        let mut server = pingora::server::Server::new_with_opt_and_conf(None, conf);
        server.bootstrap();
        let mut proxy = pingora::proxy::http_proxy_service(
            &server.configuration,
            proxy_pingora::MappingProxy { control },
        );
        proxy.add_tcp(&settings.proxy_listen);
        server.add_service(proxy);
        server.run_forever();
    }
}
