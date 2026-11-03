mod config;
mod model;
mod dto;
mod error;
mod db;
mod handler;
mod utils;
mod middleware;
mod router;
mod openapi;
use std::sync::Arc;
use axum::http::{HeaderValue, Method, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}};
use config::Config;
use db::DBClient;
use router::create_router;
use dotenv::dotenv;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;
use tokio_cron_scheduler::{JobScheduler,Job};

use crate::db::UserExt;
#[derive(Debug,Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}
#[tokio::main]
async fn main(){
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    dotenv().ok();
    let config = Config::init();
    let pool: Pool<Postgres> = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await {
            Ok(pool)=>{
                println!("Connection to the database is succesfull");
                pool
            }
            Err(err)=>{
                println!("Failed to connect to the database:{:?}",err);
                std::process::exit(1);
            }
        };
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION,ACCEPT,CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);
    let db_client = DBClient::new(pool);
    let app_state = Arc::new(AppState {
        env: config.clone(),
        db_client: db_client.clone(),
    });
    let scheduler = JobScheduler::new().await.unwrap();
    let job = Job::new_async("0 0 * * * *",{
        move |_, _| {
            let db_client = db_client.clone();
            Box::pin(async move {
                println!("Running scheduled task to delete expired files...");
                if let Err(err) = db_client.delete_expired_files().await {
                    eprintln!("Error deleting expired files: {:?}",err);
                }else {
                    println!("Successfully deleted expired files");
                }
            })
        }
    }).unwrap();
    scheduler.add(job).await.unwrap();
    tokio::spawn(async move {
        scheduler.start().await.unwrap();
    });
    let app = create_router(app_state).layer(cors);
    println!(
        "{}",
        format!("Server is running on http://localhost:{}",config.port)
    );
    println!(
        "API docs available at http://localhost:{}/swagger-ui",
        config.port
    );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    
}