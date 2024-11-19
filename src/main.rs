use std::fs::File;
use std::io::BufReader;

use config::{ConfigError, Environment};
use deadpool_postgres::{Client, Config as PoolConfig, Pool, PoolError, Runtime};
use dotenvy::dotenv;
use ntex::web::{self, App, HttpResponse, WebResponseError};
use rustls::{ClientConfig as RustlsClientConfig, RootCertStore};
use serde::{Deserialize, Serialize};
use tokio_postgres::error::Error as PGError;
use tokio_postgres_rustls::MakeRustlsConnect;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct Config {
    listen: String,
    pg: PoolConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(Environment::default().separator("__"))
            .build()
            .unwrap()
            .try_deserialize()
    }
}

#[derive(Deserialize, Serialize)]
struct Event {
    id: Uuid,
    title: String,
}

#[derive(Debug, thiserror::Error)]
enum DatabaseError {
    #[error("An internal error occurred. Please try again later.")]
    PGError(#[from] PGError),

    #[error("An internal error occurred. Please try again later.")]
    PoolError(#[from] PoolError),
}

impl WebResponseError for DatabaseError {}

async fn event_list(pool: &Pool) -> Result<Vec<Event>, PoolError> {
    let client: Client = pool.get().await?;
    let stmt = client
        .prepare_cached("SELECT id, title FROM event")
        .await?;
    let rows = client.query(&stmt, &[]).await?;
    Ok(rows
        .into_iter()
        .map(|row| Event {
            id: row.get(0),
            title: row.get(1),
        })
        .collect())
}

#[web::get("/v1.0/event.list")]
async fn index(
    db_pool: web::types::State<Pool>,
) -> Result<HttpResponse, DatabaseError> {
    let events = event_list(&db_pool).await?;
    Ok(HttpResponse::Ok().json(&events))
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::from_env().unwrap();

    // No Tls
    // let pool = config
    //     .pg
    //     .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
    //     .unwrap();

    // Rustls
    let ca_cert = "ca-certificates/cert.pem";
    let cert_file = File::open(ca_cert)?;
    let mut buf = BufReader::new(cert_file);
    let mut root_store = RootCertStore::empty();
    for cert in rustls_pemfile::certs(&mut buf) {
        root_store.add(cert?).unwrap();
    }

    let tls_config = RustlsClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let tls = MakeRustlsConnect::new(tls_config);

    let pool = config.pg.create_pool(Some(Runtime::Tokio1), tls).unwrap();

    let server = web::server(move || App::new().state(pool.clone()).service(index))
        .bind(&config.listen)?
        .run();
    println!("Server running at http://{}/", &config.listen);
    println!(
        "Try the following URLs: http://{}/v1.0/event.list",
        &config.listen,
    );

    server.await
}
