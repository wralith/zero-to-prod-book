use sqlx::PgPool;
use std::net::TcpListener;
use zero_to_prod::config::get_config;
use zero_to_prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_config().expect("Failed to read configuration");
    let pool = PgPool::connect(&configuration.database.conn_str())
        .await
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.http_port))
        .expect("Failed to bind address");

    run(listener, pool)?.await
}
