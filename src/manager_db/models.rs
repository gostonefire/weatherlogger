use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Serialize)]
pub struct DataItem<T> {
    pub x: DateTime<Local>,
    pub y: T,
}

#[derive(Serialize)]
pub struct TwoDaysMinMax {
    pub yesterday_min: f64,
    pub yesterday_max: f64,
    pub today_min: f64,
    pub today_max: f64,
}
