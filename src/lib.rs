use {
    crate::{env::Config, metrics::Metrics, project::Registry},
    anyhow::Context,
    axum::{
        http,
        response::Response,
        routing::{any, get},
        Router,
    },
    env::{BinanceConfig, ZKSyncConfig},
    error::RpcResult,
    hyper::{header::HeaderName, Client},
    hyper_tls::HttpsConnector,
    opentelemetry::metrics::MeterProvider,
    providers::{
        BinanceProvider,
        InfuraProvider,
        PoktProvider,
        ProviderRepository,
        ZKSyncProvider,
    },
    std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
        time::Duration,
    },
    tokio::{select, sync::broadcast},
    tower::ServiceBuilder,
    tower_http::{
        cors::{Any, CorsLayer},
        trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    },
    tracing::{info, Level, Span},
};

mod analytics;
pub mod env;
pub mod error;
mod extractors;
mod handlers;
mod json_rpc;
mod metrics;
mod project;
mod providers;
mod state;
mod storage;
mod utils;

pub async fn bootstrap(mut shutdown: broadcast::Receiver<()>, config: Config) -> RpcResult<()> {
    let prometheus_exporter = opentelemetry_prometheus::exporter().init();
    let meter = prometheus_exporter
        .provider()
        .unwrap()
        .meter("rpc-proxy", None);

    let metrics = Metrics::new(&meter);
    let registry = Registry::new(&config.registry, &config.storage, &meter)?;
    let providers = init_providers(&config);

    let external_ip = config
        .server
        .external_ip()
        .unwrap_or_else(|_| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

    let analytics = analytics::RPCAnalytics::new(&config.analytics, external_ip)
        .await
        .context("failed to init analytics")?;

    let state = state::new_state(
        config,
        providers,
        prometheus_exporter,
        metrics.clone(),
        registry,
        analytics,
    );

    let port = state.config.server.port;
    let host = state.config.server.host.clone();
    let build_version = state.compile_info.build().version();

    let state_arc = Arc::new(state);

    let cors = CorsLayer::new().allow_origin(Any).allow_headers([
        http::header::CONTENT_TYPE,
        http::header::USER_AGENT,
        http::header::REFERER,
        http::header::ORIGIN,
        http::header::ACCESS_CONTROL_REQUEST_METHOD,
        http::header::ACCESS_CONTROL_REQUEST_HEADERS,
        HeaderName::from_static("solana-client"),
        HeaderName::from_static("sec-fetch-mode"),
    ]);

    let global_middleware = ServiceBuilder::new().layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .include_headers(true),
            ),
    );

    let proxy_state = state_arc.clone();
    let proxy_metrics = ServiceBuilder::new().layer(TraceLayer::new_for_http().on_response(
        move |response: &Response, latency: Duration, _span: &Span| {
            proxy_state
                .metrics
                .add_http_call(response.status().into(), "proxy");

            proxy_state.metrics.add_http_latency(
                response.status().into(),
                "proxy",
                latency.as_secs_f64(),
            )
        },
    ));

    let app = Router::new()
        .route("/v1", any(handlers::proxy::handler))
        .route_layer(proxy_metrics)
        .route("/health", get(handlers::health::handler))
        .route("/metrics", get(handlers::metrics::handler))
        .layer(cors)
        .layer(global_middleware)
        .with_state(state_arc.clone());

    info!("v{}", build_version);
    info!("Running RPC Proxy on port {}", port);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("Invalid socket address");

    select! {
    _ = shutdown.recv() => info!("Shutdown signal received, killing servers"),
    _ = axum::Server::bind(&addr).serve(app.into_make_service_with_connect_info::<SocketAddr>()) => info!("Server terminating")
        }
    Ok(())
}

fn init_providers(config: &Config) -> ProviderRepository {
    let infura_project_id = config.infura.project_id.clone();
    let infura_supported_chains = config.infura.supported_chains.clone();
    let pokt_project_id = config.pokt.project_id.clone();
    let pokt_supported_chains = config.pokt.supported_chains.clone();

    let mut providers = ProviderRepository::default();
    let forward_proxy_client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());

    let infura_provider = InfuraProvider {
        client: forward_proxy_client.clone(),
        project_id: infura_project_id,
        supported_chains: infura_supported_chains,
    };
    providers.add_provider("infura".into(), Arc::new(infura_provider));

    let pokt_provider = PoktProvider {
        client: forward_proxy_client.clone(),
        project_id: pokt_project_id,
        supported_chains: pokt_supported_chains,
    };
    providers.add_provider("pokt".into(), Arc::new(pokt_provider));

    let binance_config = BinanceConfig::default();
    let binance_provider = BinanceProvider {
        client: forward_proxy_client.clone(),
        project_id: binance_config.project_id,
        supported_chains: binance_config.supported_chains,
    };
    providers.add_provider("binance".into(), Arc::new(binance_provider));

    let zksync_config = ZKSyncConfig::default();
    let zksync_provider = ZKSyncProvider {
        client: forward_proxy_client,
        project_id: zksync_config.project_id,
        supported_chains: zksync_config.supported_chains,
    };
    providers.add_provider("zksync".into(), Arc::new(zksync_provider));

    providers
}
