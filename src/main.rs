use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero_to_prod::config::get_config;
use zero_to_prod::startup::run;
use zero_to_prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero_to_prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_config().expect("Failed to read configuration");
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.http_port
    ))
    .expect("Failed to bind address");

    run(listener, pool)?.await?;
    Ok(())
}
