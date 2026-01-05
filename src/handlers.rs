use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use log::{error, info};
use serde::Deserialize;
use crate::SharedState;


#[derive(Deserialize, Debug)]
pub struct TempParams {
    id: String,
    from: String,
    to: String,
}

#[derive(Deserialize, Debug)]
pub struct MinMaxParams {
    id: String,
    from: String,
    to: String,
}

pub async fn temperature(Query(params): Query<TempParams>, State(state): State<SharedState>) -> impl IntoResponse {
    info!("temperature: {:?}", params);

    let db = state.lock().await;

    match db.get_temp_history(&params.id, &params.from, &params.to) {
        Ok(json) => ([(header::CONTENT_TYPE, "application/json")], json).into_response(),
        Err(e) => {
            error!("failed to get temp history: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn min_max(Query(params): Query<MinMaxParams>, State(state): State<SharedState>) -> impl IntoResponse {
    info!("minmax: {:?}", params);

    let db = state.lock().await;

    match db.get_min_max(&params.id, &params.from, &params.to) {
        Ok(json) => ([(header::CONTENT_TYPE, "application/json")], json).into_response(),
        Err(e) => {
            error!("failed to get min/max: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn forecast(Query(params): Query<TempParams>, State(state): State<SharedState>) -> impl IntoResponse {
    info!("forecast: {:?}", params);
    
    let db = state.lock().await;
    
    match db.get_forecast(&params.id, &params.from, &params.to) {
        Ok(json) => ([(header::CONTENT_TYPE, "application/json")], json).into_response(),
        Err(e) => {
            error!("failed to get forecast: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}