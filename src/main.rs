#[forbid(unsafe_code)]
#[deny(missing_docs, unused_mut)]
mod crypto;
mod database;
mod metrics;
mod router;
mod status;
mod user;
mod well_known;

use axum::{
    http::{header, Method},
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::RequestBodyTimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use well_known::well_known;

use std::env;
use std::future::ready;
use std::process;
use std::time::Duration;

#[derive(Clone)]
pub struct AppState {
    pub config: status::Configuration,
    pub db: database::Database,
}

/// Create router.
pub fn app(state: AppState) -> Router {
    Router::new()
        // `GET /status.json` goes to `status`.
        .route("/status.json", get(router::status::status))
        // `POST /login` goes to `login`.
        .route("/login", post(router::login::login))
        // `POST /create` goes to `create`.
        .route("/create", post(router::create::create))
        // `GET /users/:USER_ID` goes to `get`.
        .route("/users/{user_id}", get(router::user::get))
        .with_state(state.clone())
        .nest("/.well-known", well_known(state))
        .layer(TraceLayer::new_for_http())
        .route_layer(middleware::from_fn(metrics::track_metrics))
        .route_layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH])
                .vary([header::AUTHORIZATION]),
        )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing.
    let (layer, task) = tracing_loki::builder()
        .label("host", "autha")?
        .extra_field("pid", process::id().to_string())?
        .build_url(url::Url::parse("http://loki:3100")?)?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(layer)
        .with(tracing_subscriber::fmt::layer())
        .init();

    tokio::spawn(task); // init Loki delivrerer.

    // init all telemetry logic.
    let recorder_handle = metrics::setup_metrics_recorder()?;

    // load configuration and let it on memory.
    // init databases connection.
    let state = AppState {
        config: status::Configuration::read(None)?,
        db: database::Database::new(
            &env::var("POSTGRES_URL").unwrap_or_else(|_| database::DEFAULT_PG_URL.into()),
        )
        .await?,
    };

    // build our application with a route.
    let app = app(state)
        // `GET /metrics`
        .route("/metrics", get(move || ready(recorder_handle.render())))
        .layer(RequestBodyTimeoutLayer::new(Duration::from_secs(5)));

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or(8080.to_string())
    ))
    .await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
