use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Deserialize)]
pub struct Data {
    pub air_temperature: f64,
    pub wind_speed: f64,
    pub relative_humidity: u8,
    pub low_type_cloud_area_fraction: u8,
    pub medium_type_cloud_area_fraction: u8,
    pub high_type_cloud_area_fraction: u8,
    pub symbol_code: f64,
}


#[derive(Deserialize)]
pub struct FullTimeSeries {
    pub time: DateTime<Utc>,
    pub data: Data,
}


#[derive(Deserialize)]
pub struct FullForecast {
    #[serde(rename = "timeSeries")]
    pub time_series: Vec<FullTimeSeries>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct ForecastValues {
    pub valid_time: DateTime<Utc>,
    pub temp: f64,
    pub wind_speed: f64,
    pub relative_humidity: u8,
    pub lcc_mean: u8,
    pub mcc_mean: u8,
    pub hcc_mean: u8,
    pub symbol_code: u8,
}