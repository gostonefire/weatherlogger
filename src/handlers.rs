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
struct QueryParams {
    id: String,
    from: String,
    to: String,
}

#[get("/log")]
async fn log_data(params: web::Query<SensorData>, data: web::Data<AppState>) -> impl Responder {
    info!("{:?}", params);

    let db = data.db.lock().await;

    match db.insert_record(&params.id, params.temp, params.hum) {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!("Failed to insert record: {}", e);
            HttpResponse::InternalServerError().finish()
        },
    }
}

#[get("/temperature")]
async fn temperature(params: web::Query<QueryParams>, data: web::Data<AppState>) -> impl Responder {
    info!("{:?}", params);

    let db = data.db.lock().await;

    match db.get_temp_history(&params.id, &params.from, &params.to) {
        Ok(json) => HttpResponse::Ok().body(json),
        Err(e) => {
            error!("failed to get temp history: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
