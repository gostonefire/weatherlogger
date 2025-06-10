mod errors;
mod logging;
mod initialization;

use std::sync::Arc;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use serde::Deserialize;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use log::info;
use crate::errors::UnrecoverableError;
use crate::initialization::config;

struct AppState {
    db_conn: Arc<Mutex<Connection>>,
}

#[derive(Deserialize, Debug)]
struct Data {
    hum: u8,
    temp: f64,
}

// hum=60&temp=25.00&id=shellyht-xxxxxx
#[get("/log")]
async fn log_data(params: web::Query<Data>, data: web::Data<AppState>) -> impl Responder {
    info!("{:?}", params);
    
    let now = Utc::now().timestamp();
    let db_conn = data.db_conn.lock().await;

    match db_conn.execute(
        "INSERT INTO temp_hum (datetime, temperature, humidity) values (?1, ?2, ?3)",
        params![now, params.temp, params.hum],
    ) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> Result<(), UnrecoverableError> {
    let config = config()?;
    let conn = Connection::open(config.db.db_path)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS temp_hum (
                datetime integer primary key,
                temperature real null,
                humidity integer null
         )",
        [],
    )?;
    
    let db_conn: Arc<Mutex<Connection>> = Arc::new(Mutex::new(conn));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {db_conn: db_conn.clone()}))
            .service(log_data)
    })
        .bind((config.web_server.bind_address, config.web_server.bind_port))?
        .run()
        .await?;
    
    Ok(())
}
