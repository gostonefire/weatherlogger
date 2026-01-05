mod errors;
mod logging;
mod initialization;
mod manager_db;
mod handlers;
mod manager_temperature;
mod manager_smhi;
mod manager_forecast;
mod perceived_temperature;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use log::error;
use tokio::sync::Mutex;
use crate::errors::UnrecoverableError;
use crate::handlers::{forecast, min_max, temperature};
use crate::initialization::config;
use crate::manager_db::DB;
use crate::manager_forecast::run_forecasts;
use crate::manager_temperature::run_observations;

pub type SharedState = Arc<Mutex<DB>>;

#[tokio::main]
async fn main() -> Result<(), UnrecoverableError> {
    let config = config()?;
    let state: SharedState = Arc::new(Mutex::new(DB::new(&config.db.db_path, config.db.max_age_in_days)?));

    let c1_db = state.clone();
    tokio::spawn(async move {
        loop {
            {
                c1_db.lock().await.truncate_table();
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(86400)).await;
        }
    });

    let c2_db = state.clone();
    tokio::spawn(async move {
        run_observations(c2_db, &config.temperature.sensor, &config.temperature.name).await;
    });

    let c3_db = state.clone();
    tokio::spawn(async move {
        run_forecasts(c3_db, config.weather_forecast.lat, config.weather_forecast.long, &config.weather_forecast.name).await;
    });

    let app = Router::new()
        .route("/temperature", get(temperature))
        .route("/minmax", get(min_max))
        .route("/forecast", get(forecast))
        .with_state(state.clone());

    let ip_addr = Ipv4Addr::from_str(&config.web_server.bind_address).expect("invalid BIND_ADDR");
    let addr = SocketAddr::new(IpAddr::V4(ip_addr), config.web_server.bind_port);

    let result = axum_server::bind(addr)
        .serve(app.into_make_service())
        .await;

    if let Err(e) = result {
        error!("server error: {}", e);
        Err(UnrecoverableError(format!("server error: {}", e)))?
    } else {
        Ok(())
    }
}