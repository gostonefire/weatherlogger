use actix_web::{get, web, HttpResponse, Responder};
use log::{error, info};
use serde::Deserialize;
use crate::AppState;

#[derive(Deserialize, Debug)]
struct SensorData {
    hum: u8,
    temp: f64,
    id: String,
}

#[derive(Deserialize, Debug)]
struct TempParams {
    id: String,
    from: String,
    to: String,
}

#[derive(Deserialize, Debug)]
struct MinMaxParams {
    id: String,
}

#[get("/log")]
async fn log_data(params: web::Query<SensorData>, data: web::Data<AppState>) -> impl Responder {
    info!("{:?}", params);

    let db = data.db.lock().await;

    match db.insert_observation_record(&params.id, params.temp, Some(params.hum)) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!("Failed to insert record: {}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

#[get("/temperature")]
async fn temperature(params: web::Query<TempParams>, data: web::Data<AppState>) -> impl Responder {
    info!("temperature: {:?}", params);

    let db = data.db.lock().await;

    match db.get_temp_history(&params.id, &params.from, &params.to) {
        Ok(json) => HttpResponse::Ok().body(json),
        Err(e) => {
            error!("failed to get temp history: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/minmax")]
async fn min_max(params: web::Query<MinMaxParams>, data: web::Data<AppState>) -> impl Responder {
    info!("minmax: {:?}", params);

    let db = data.db.lock().await;

    match db.get_two_day_min_max(&params.id) {
        Ok(json) => HttpResponse::Ok().body(json),
        Err(e) => {
            error!("failed to get min/max: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}