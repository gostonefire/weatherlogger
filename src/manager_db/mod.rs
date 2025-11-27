pub mod errors;
mod models;

use std::ops::Add;
use chrono::{DateTime, TimeDelta, Utc};
use log::error;
use rusqlite::{params, Connection};
use crate::manager_db::errors::DBError;
use crate::manager_db::models::{DataItem, ForecastRecord, TwoDaysMinMax};

pub struct DB {
    db_conn: Connection,
    max_age_in_days: i64,
    
}

impl DB {
    
    /// Creates a new instance of DB
    /// 
    /// # Arguments
    /// 
    /// * 'db_path' - full path to db file
    /// * 'max_age_in_days' - the limit to truncate table on
    pub fn new(db_path: &str, max_age_in_days: i64) -> Result<Self, DBError> {
        let db_conn = Connection::open(db_path)?;
        db_conn.execute(
           "CREATE TABLE IF NOT EXISTS weather (
                source text not null,
                datetime integer not null,
                temperature real null,
                humidity integer null,
                wind_speed real null,
                lcc_mean integer null,
                mcc_mean integer null,
                hcc_mean integer null,
                constraint primary_key primary key (source, datetime)
           )",
           [],
        )?;
        
        Ok(DB { db_conn, max_age_in_days })
    }
    
    /// Inserts an observation record in the database
    /// 
    /// # Arguments
    ///
    /// * 'source' - sensor id (source)    
    /// * 'temp' - temperature
    /// * 'humidity' - humidity
    pub fn insert_observation_record(
        &self,
        source: &str,
        temp: f64,
        humidity: Option<u8>,
    ) -> Result<(), DBError> {
        
        self.db_conn.execute(
            "INSERT INTO weather (source, datetime, temperature, humidity) VALUES (?1, ?2, ?3, ?4)",
            params![source, Utc::now().timestamp(), temp, humidity],
        )?;
        
        Ok(())
    }

    /// Inserts (or updates) a forecast record in the database.
    /// The main difference is that these records often tend to update existing records
    /// when newer forecasts replace older ones.
    ///
    /// # Arguments
    ///
    /// * 'source' - sensor id (source)
    /// * 'date_time' - forecast date and time in UTC
    /// * 'temp' - temperature
    /// * 'wind_speed' - wind speed
    /// * 'humidity' - humidity
    /// * 'lcc_mean' - low level cloud index
    /// * 'mcc_mean' - medium level cloud index
    /// * 'hcc_mean' - high level cloud index
    pub fn insert_forecast_record(
        &self,
        source: &str,
        date_time: DateTime<Utc>,
        temp: f64,
        wind_speed: Option<f64>,
        humidity: Option<u8>,
        lcc_mean: Option<u8>,
        mcc_mean: Option<u8>,
        hcc_mean: Option<u8>,
    ) -> Result<(), DBError> {

        self.db_conn.execute(
            "INSERT INTO weather (source, datetime, temperature, humidity, wind_speed, lcc_mean, mcc_mean, hcc_mean)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                    ON CONFLICT (source, datetime) DO UPDATE SET temperature = ?3, humidity = ?4, wind_speed = ?5, lcc_mean = ?6, mcc_mean = ?7, hcc_mean = ?8",
            params![source, date_time.timestamp(), temp, humidity, wind_speed, lcc_mean, mcc_mean, hcc_mean],
        )?;

        Ok(())
    }

    /// Returns a json string with whatever temperatures are recorded between (non-inclusive) given boundaries
    /// 
    /// Since the sensor only records data when there is a change in either temperature (1 degree Celsius) or
    /// humidity (5%), there is a chance that no data would be returned even for a longer period of time.
    /// 
    /// To mitigate this, indeed there is a temperature, the last recorded temperature will be returned with
    /// a time set to the given `from` parameter if the resultset from db was empty.
    /// 
    /// # Arguments
    /// 
    /// * 'source' - sensor id (source)
    /// * 'from' - utc datetime in the rfc3339 format
    /// * 'to' - utc datetime in the rfc3339 format (non-inclusive)
    pub fn get_temp_history(&self, source: &str, from: &str, to: &str) -> Result<String, DBError> {
        let from_datetime = DateTime::parse_from_rfc3339(from)?.with_timezone(&Utc);
        let from_timestamp = DateTime::parse_from_rfc3339(from)?.timestamp();
        let to_timestamp = DateTime::parse_from_rfc3339(to)?.timestamp();
        
        let mut result: Vec<DataItem<f64>> = Vec::new();
        
        // Get what may naturally be between the given time boundary from the database
        let mut stmt = self.db_conn.prepare(
            "SELECT datetime, temperature 
                FROM weather
                WHERE source = ?1 AND datetime >= ?2 AND datetime < ?3
                ORDER BY datetime;",
        )?;
        let mut rows = stmt.query(params![source, from_timestamp, to_timestamp])?;

        while let Some(row) = rows.next()? {
            let timestamp: i64 = row.get(0)?;
            let x = DateTime::from_timestamp(timestamp, 0).unwrap();
            let y: f64 = row.get(1)?;
            result.push(DataItem { x, y });
        }
        
        // Make sure that we have at least one data point
        if result.is_empty() {
            let mut stmt = self.db_conn.prepare(
                "SELECT temperature 
                FROM weather
                WHERE source = ?1
                ORDER BY datetime DESC LIMIT 1;",
            )?;
            let x =  from_datetime;
            let response: rusqlite::Result<f64> = stmt.query_one(params![source], |row| row.get(0));
            match response {
                Ok(y) => result.push(DataItem { x, y }),
                Err(e) => {
                    if e != rusqlite::Error::QueryReturnedNoRows {
                        return Err(DBError::from(e));
                    }
                }
            }
        }
        
        Ok(serde_json::to_string_pretty(&result)?)
    }

    /// Returns a json string with whatever forecasts are recorded between (non-inclusive) given boundaries
    ///
    /// # Arguments
    ///
    /// * 'source' - source responsible for forecast values
    /// * 'from' - utc datetime in the rfc3339 format
    /// * 'to' - utc datetime in the rfc3339 format (non-inclusive)
    pub fn get_forecast(&self, source: &str, from: &str, to: &str) -> Result<String, DBError> {
        let from_timestamp = DateTime::parse_from_rfc3339(from)?.with_timezone(&Utc).timestamp();
        let to_timestamp = DateTime::parse_from_rfc3339(to)?.with_timezone(&Utc).timestamp();

        let mut result: Vec<ForecastRecord> = Vec::new();

        // Get what may naturally be between the given time boundary from the database
        let mut stmt = self.db_conn.prepare(
            "SELECT datetime, temperature, wind_speed, humidity, lcc_mean, mcc_mean, hcc_mean
                FROM weather
                WHERE source = ?1 AND datetime >= ?2 AND datetime < ?3
                ORDER BY datetime;",
        )?;
        let mut rows = stmt.query(params![source, from_timestamp, to_timestamp])?;

        while let Some(row) = rows.next()? {
            let timestamp: i64 = row.get(0)?;
            let fc = ForecastRecord {
                date_time: DateTime::from_timestamp(timestamp, 0).unwrap(),
                temperature: row.get(1)?,
                wind_speed: row.get(2)?,
                humidity: row.get(3)?,
                lcc_mean: row.get(4)?,
                mcc_mean: row.get(5)?,
                hcc_mean: row.get(6)?,
            };

            result.push(fc);
        }

        Ok(serde_json::to_string_pretty(&result)?)
    }

    /// Returns a json string with two-day min/max temperature values
    ///
    /// # Arguments
    ///
    /// * 'source' - sensor id (source)
    /// * 'from' - utc datetime in the rfc3339 format
    /// * 'to' - utc datetime in the rfc3339 format (non-inclusive)
    pub fn get_two_day_min_max(&self, source: &str, from: &str, to: &str) -> Result<String, DBError> {
        let today_start = DateTime::parse_from_rfc3339(from)?.with_timezone(&Utc);
        let today_end = DateTime::parse_from_rfc3339(to)?.with_timezone(&Utc);
        let yesterday_start = today_start.add(TimeDelta::days(-1));
        let yesterday_end = today_end.add(TimeDelta::days(-1));

        // Get min/max
        let mut stmt = self.db_conn.prepare(
            "SELECT MIN(temperature), MAX(temperature) 
                FROM weather
                WHERE source = ?1 AND datetime >= ?2 AND datetime < ?3;",
        )?;
        

        let mut result = TwoDaysMinMax {
            yesterday_min: 0.0,
            yesterday_max: 0.0,
            today_min: 0.0,
            today_max: 0.0,
        };

        {
            let rows = &mut stmt.query(params![source, yesterday_start.timestamp(), yesterday_end.timestamp()])?;
            if let Some(row) = rows.next()? {
                result.yesterday_min = row.get(0).unwrap_or(0.0);
                result.yesterday_max = row.get(1).unwrap_or(0.0);
            }
        }

        {
            let rows = &mut stmt.query(params![source, today_start.timestamp(), today_end.timestamp()])?;
            if let Some(row) = rows.next()? {
                result.today_min = row.get(0).unwrap_or(0.0);
                result.today_max = row.get(1).unwrap_or(0.0);
            }
        }

        Ok(serde_json::to_string_pretty(&result)?)
    }
    
    /// Truncate weather table according max age
    /// 
    pub fn truncate_table(&self) {
        match self.db_conn.prepare(
            "DELETE FROM weather
                WHERE datetime < ?1;"
        ) {
            Ok(mut stmt) => { 
                let trunc_time = Utc::now().add(TimeDelta::days(-1 * self.max_age_in_days)).timestamp();
                if let Err(e) = stmt.query(params![trunc_time]) {
                    error!("error while deleting rows: {}", e);
                }
            },
            Err(e) => { error!("error while preparing delete statement: {}", e); }
        }
    }
}