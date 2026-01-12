pub mod errors;
mod models;

use std::ops::Add;
use std::time::Duration;
use chrono::{DateTime, DurationRound, Utc, TimeDelta};
use reqwest::Client;
use crate::manager_smhi::errors::SMHIError;
use crate::manager_smhi::models::{ForecastValues, FullForecast};


/// Struct for managing whether forecasts produced by SMHI
pub struct SMHI {
    client: Client,
    lat: f64,
    long: f64,
}

impl SMHI {
    /// Returns a SMHI struct ready for fetching and processing whether forecasts from SMHI
    ///
    /// The given lat/long values will be truncated to 4 decimals since that is the max
    /// precision that SMHI allows in their forecast API
    ///
    /// # Arguments
    ///
    /// * 'lat' - latitude of the location
    /// * 'long' - longitude of the location
    pub fn new(lat: f64, long: f64) -> Result<SMHI, SMHIError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            lat,
            long,
        })
    }

    /// Retrieves a weather forecast from SMHI for the given date.
    /// The raw forecast consists of several days worth of data and many weather parameters,
    /// but the returned forecast will only include the specified date (plus one) and data
    /// representing cloud index (0-8) and forecasted temperatures.
    ///
    /// # Arguments
    ///
    /// * 'date_time' - the date to get a forecast for
    pub async fn new_forecast(&self, date_time: DateTime<Utc>) -> Result<Vec<ForecastValues>, SMHIError> {
        let smhi_domain = "https://opendata-download-metfcst.smhi.se";
        let base_url = "/api/category/snow1g/version/1/geotype/point";
        let url = format!("{}{}/lon/{:0.4}/lat/{:0.4}/data.json",
                          smhi_domain, base_url, self.long, self.lat);

        let date = date_time.duration_trunc(TimeDelta::days(1)).unwrap();
        let next_date = date.add(TimeDelta::days(1));

        let req = self.client
            .get(url)
            .send().await?;

        let status = req.status();
        if !status.is_success() {
            return Err(SMHIError::SMHI(format!("Error while fetching forecast from SMHI: {}", status)));
        }

        let json = req.text().await?;
        let tmp_forecast: FullForecast = serde_json::from_str(&json)?;

        let mut forecast: Vec<ForecastValues> = Vec::new();

        for ts in tmp_forecast.time_series {
            let forecast_date = ts.time.duration_trunc(TimeDelta::days(1)).unwrap();
            if forecast_date == date || forecast_date == next_date {
                let time_values = ForecastValues {
                    valid_time: ts.time,
                    temp: ts.data.air_temperature,
                    wind_speed: ts.data.wind_speed,
                    relative_humidity:ts.data.relative_humidity,
                    lcc_mean: ts.data.low_type_cloud_area_fraction,
                    mcc_mean: ts.data.medium_type_cloud_area_fraction,
                    hcc_mean: ts.data.high_type_cloud_area_fraction,
                    symbol_code: ts.data.symbol_code.round() as u8,
                };

                forecast.push(time_values);
            }
        }
       
        if forecast.len() == 0 {
            Err(SMHIError::SMHI(format!("No forecast found for {}", date_time.date_naive())))
        } else {
            Ok(forecast)
        }
    }
}
