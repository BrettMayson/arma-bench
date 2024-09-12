use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    require_env("STEAM_USER");
    require_env("STEAM_PASS");

    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_target(false)
        .compact();

    let stdout = tracing_subscriber::fmt::layer().event_format(format);

    tracing_subscriber::registry().with(stdout).init();

    let addr = std::env::var("TAB_ADDR")
        .unwrap_or_else(|_| format!("0.0.0.0:{}", arma_bench::DEFAULT_PORT));
    arma_bench_server::server(addr).await;
}

fn require_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} not set"))
}
