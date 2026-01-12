use std::sync::Arc;
use chrono::Utc;
use log::error;
use tokio::sync::Mutex;
use crate::manager_db::DB;
use crate::manager_smhi::SMHI;

/// Forecast reading loop
///
/// # Arguments
///
/// * 'db' - database to store readings into
/// * 'lat' - latitude of the location
/// * 'long' - longitude of the location
/// * 'name' - the name of the forecast provider
pub async fn run_forecasts(db: Arc<Mutex<DB>>, lat: f64, long: f64, name: &str) {
    let smhi = if let Ok(smhi) = SMHI::new(lat, long) {
        smhi
    } else {
        error!("failed to create SMHI manager");
        return;
    };

    loop {
        if let Ok(forecast) = smhi.new_forecast(Utc::now()).await {
            for f in forecast {
                if let Err(e) = db.lock().await.insert_forecast_record(
                    name,
                    f.valid_time,
                    f.temp,
                    Some(f.wind_speed),
                    Some(f.relative_humidity),
                    Some(f.lcc_mean),
                    Some(f.mcc_mean),
                    Some(f.hcc_mean),
                    Some(f.symbol_code),
                ) {
                    error!("failed to insert forecast record: {}", e);
                }
            }
        } else {
            error!("failed to get forecast from SMHI");
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}