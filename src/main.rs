mod errors;
mod logging;
mod initialization;
mod manager_db;
mod handlers;

use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use tokio::sync::Mutex;
use crate::errors::UnrecoverableError;
use crate::handlers::{log_data, min_max, temperature};
use crate::initialization::config;
use crate::manager_db::DB;

struct AppState {
    db: Arc<Mutex<DB>>,
}


#[actix_web::main]
async fn main() -> Result<(), UnrecoverableError> {
    let config = config()?;
    let db: Arc<Mutex<DB>> = Arc::new(Mutex::new(DB::new(&config.db.db_path, config.db.max_age_in_days)?));

    let trunc = db.clone();
    tokio::spawn(async move {
        loop {
            {
                trunc.lock().await.truncate_table();
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
        }
    });
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {db: db.clone()}))
            .service(log_data)
            .service(temperature)
            .service(min_max)
    })
        .bind((config.web_server.bind_address, config.web_server.bind_port))?
        .run()
        .await?;
    
    Ok(())
}
