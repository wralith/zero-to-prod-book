use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero_to_prod::{
    config::{get_config, DatabaseSettings},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "info".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test".into(), "info".into(), std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    // Port 0 means: Hey os give me some random empty port
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let mut config = get_config().expect("Failed to read configugration");
    config.database.database_name = Uuid::new_v4().to_string();
    let pool = configure_database(&config.database).await;
    // Idk what I am doing

    let server = zero_to_prod::startup::run(listener, pool.clone()).expect("Unable to bind");
    let _ = tokio::spawn(server);

    TestApp { address, pool }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut conn = PgConnection::connect(&config.conn_str_without_db())
        .await
        .expect("Failed to connect to Postgres");
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let pool = PgPool::connect(&config.conn_str())
        .await
        .expect("Failed to connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate");

    pool
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Falied to execute request");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_input() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=test&email=test%40test.com";

    let res = client
        .post(format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, res.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "test@test.com");
    assert_eq!(saved.name, "test");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let cases = vec![
        ("name=wra%20lith", "missing email"),
        ("email=wralithdev%40gmail.com", "missing name"),
        ("", "missing name and email"),
    ];

    for (body, error_message) in cases {
        let res = client
            .post(format!("{}/subscriptions", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            res.status().as_u16(),
            "API did not fail with 400 when the payload was {error_message}"
        );
    }
}
