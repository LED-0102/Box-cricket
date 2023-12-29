mod auth;

use std::env;
use actix_web::{web, HttpResponse, Responder, HttpServer, App};
use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::middleware::Logger;
use dotenv::dotenv;
use crate::auth::auth_config;
use env_logger::{Env};
use tokio_postgres::{Client, NoTls};

pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

pub struct Database {
    client: Client
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    dotenv().ok();
    let user = env::var("USER").expect("User not set in .env");
    let password = env::var("PASSWORD").expect("Password not set for User in .env");
    let dbname = env::var("DBNAME").expect("Database name not set in .env");

    let config = format!("host=localhost user={} password={} dbname = {}", user, password, dbname);
    let (client, connection) = tokio_postgres::connect(
        &config, NoTls
    ).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let db = web::Data::new( Database {
        client,
    });
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::default().allow_any_origin().allow_any_method().allow_any_header())
            .wrap_fn(|req, srv| {
                let future = srv.call(req);
                async {
                    let result = future.await?;
                    Ok(result)
                }
            })
            .configure(auth_config)
            .app_data(db.clone())
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("/") }),
            )
    })
        .bind(("127.0.0.1", 8080))?
        .workers(1)
        .run()
        .await

}