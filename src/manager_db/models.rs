use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct DataItem<T> {
    pub x: DateTime<Utc>,
    pub y: T,
}

#[derive(Serialize)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}

#[derive(Serialize)]
pub struct ForecastRecord {
    pub date_time: DateTime<Utc>,
    pub temperature: Option<f64>,
    pub wind_speed: Option<f64>,
    pub humidity: Option<u8>,
    pub lcc_mean: Option<u8>,
    pub mcc_mean: Option<u8>,
    pub hcc_mean: Option<u8>,
}