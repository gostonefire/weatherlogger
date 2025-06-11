use chrono::{DateTime, Local};
use serde::Serialize;

#[derive(Serialize)]
pub struct DataItem<T> {
    pub x: DateTime<Local>,
    pub y: T,
}