use std::sync::Arc;
use log::{error, info};
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use crate::errors::TempError;
use crate::manager_db::DB;

#[derive(Deserialize)]
struct Data {
    data: f64,
}

/// Sensors reading loop
///
/// # Arguments
///
/// * 'db' - database to store readings into
/// * 'sensor' - a vector of sensors to read
/// * 'name' - the name of the sensor
pub async fn run_observations(db: Arc<Mutex<DB>>, sensor: &Vec<String>, name: &str) {
    let mut last_inserted: f64 = 0.0;

    loop {
        let mut set: JoinSet<Result<f64, TempError>> = JoinSet::new();

        for s in sensor.iter() {
            let url = s.clone();
            set.spawn(async move { request_data(url).await });
        }

        let result = set.join_all().await;
        let mut temperature: Option<f64> = None;
        for reading in result.into_iter() {
            if let Ok(r) = reading {
                info!("temperature: {}", r);
                match &mut temperature {
                    Some(t) => *t = t.min(r),
                    t @ _ => *t = Some(r),
                }
            }
        }

        if let Some(t) = temperature {
            if t != last_inserted {
                if let Err(e) = db.lock().await.insert_observation_record(name, t, None) {
                    error!("error while inserting data in database: {}", e);
                }
                last_inserted = t;

                info!("inserted {} in database", t);
            }

        }

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}

/// Makes a request for temperature data from one sensor
///
/// # Arguments
///
/// * 'url' - url to sensor
async fn request_data(url: String) -> Result<f64, TempError> {
    let response = reqwest::get(&url).await?;
    if response.status().is_success() {
        let json = response.text().await?;
        let data: Data = serde_json::from_str(&json)?;
        Ok(data.data)
    } else {
        Err(TempError(format!("status code: {}", response.status())))
    }
}