use axum::http::Method;
use std::net::TcpListener;
use std::sync::Arc;

use crate::api;
use crate::api::HandlerContext;
use crate::config::Config;
use crate::layers::HttpLoggerLayer;

pub struct Application {
    port: u16,
    server: axum::Server<
        hyper::server::conn::AddrIncoming,
        axum::routing::IntoMakeService<axum::Router>,
    >,
}

impl Application {
    pub async fn build(config: Config) -> anyhow::Result<Self> {
        let addr = (config.addr)
            .clone()
            .unwrap_or_else(|| "127.0.0.1:8080".to_string());
        let listener = TcpListener::bind(addr)?;
        let port = listener.local_addr().unwrap().port();

        let handler_ctx = HandlerContext::new(&config)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        let routes = api::routes::register_app_routes();
        let cors = tower_http::cors::CorsLayer::new()
            .allow_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
            .allow_origin(tower_http::cors::Any)
            .allow_credentials(false);
        let mut app = routes
            .layer(axum::AddExtensionLayer::new(Arc::new(handler_ctx)))
            .layer(cors);
        if log::log_enabled!(log::Level::Info) {
            // only add logger layer if level >= INFO
            app = app.layer(HttpLoggerLayer::new());
        }
        let server = axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service());

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> anyhow::Result<()> {
        Ok(self.server.await?)
    }
}
