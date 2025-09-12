mod errors;
mod logging;
mod initialization;
mod manager_db;
mod handlers;
mod manager_temperature;
mod manager_smhi;
mod manager_forecast;

use std::sync::Arc;
use actix_web::{web, App, HttpServer};
use tokio::sync::Mutex;
use crate::errors::UnrecoverableError;
use crate::handlers::{forecast, log_data, min_max, temperature};
use crate::initialization::config;
use crate::manager_db::DB;
use crate::manager_forecast::run_forecasts;
use crate::manager_temperature::run_observations;

struct AppState {
    db: Arc<Mutex<DB>>,
}


#[actix_web::main]
async fn main() -> Result<(), UnrecoverableError> {
    let config = config()?;
    let db: Arc<Mutex<DB>> = Arc::new(Mutex::new(DB::new(&config.db.db_path, config.db.max_age_in_days)?));

    let c1_db = db.clone();
    tokio::spawn(async move {
        loop {
            {
                c1_db.lock().await.truncate_table();
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
        }
    });

    let c2_db = db.clone();
    tokio::spawn(async move {
        run_observations(c2_db, &config.temperature.sensor, &config.temperature.name).await;
    });

    let c3_db = db.clone();
    tokio::spawn(async move {
        run_forecasts(c3_db, config.weather_forecast.lat, config.weather_forecast.long, &config.weather_forecast.name).await;
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {db: db.clone()}))
            .service(log_data)
            .service(temperature)
            .service(min_max)
            .service(forecast)
    })
        .bind((config.web_server.bind_address, config.web_server.bind_port))?
        .run()
        .await?;
    
    Ok(())
}
