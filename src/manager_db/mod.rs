pub mod errors;
mod models;

use std::ops::Add;
use chrono::{DateTime, TimeDelta, Utc};
use log::error;
use rusqlite::{params, Connection};
use crate::manager_db::errors::DBError;
use crate::manager_db::models::{DataItem, ForecastRecord, MinMax, Temperature};

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
                perceived_temperature real null,
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
        perceived_temp: Option<f64>,
    ) -> Result<(), DBError> {
        
        self.db_conn.execute(
            "INSERT INTO weather (source, datetime, temperature, humidity, perceived_temperature) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![source, Utc::now().timestamp(), temp, humidity, perceived_temp],
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
        
        let mut result = Temperature {
            history: Vec::new(),
            current_temp: None,
            perceived_temp: None,
        };
        
        // Get what may naturally be between the given time boundary from the database
        let mut stmt = self.db_conn.prepare(
            "SELECT datetime, temperature, perceived_temperature 
                FROM weather
                WHERE source = ?1 AND datetime >= ?2 AND datetime < ?3
                ORDER BY datetime;",
        )?;
        let mut rows = stmt.query(params![source, from_timestamp, to_timestamp])?;

        while let Some(row) = rows.next()? {
            let timestamp: i64 = row.get(0)?;
            let x = DateTime::from_timestamp(timestamp, 0).unwrap();
            let y: f64 = row.get(1)?;
            result.current_temp = Some(y);
            result.perceived_temp = row.get(2)?;
            result.history.push(DataItem { x, y });
        }
        
        // Make sure that we have at least one data point
        if result.history.is_empty() {
            let mut stmt = self.db_conn.prepare(
                "SELECT temperature, perceived_temperature
                FROM weather
                WHERE source = ?1
                ORDER BY datetime DESC LIMIT 1;",
            )?;
            let x =  from_datetime;
            let response: rusqlite::Result<(f64,Option<f64>)> = stmt.query_one(params![source], |row| {
                Ok((row.get(0)?, row.get(1)?))
            });
            match response {
                Ok(y) => {
                    result.current_temp = Some(y.0);
                    result.perceived_temp = y.1;
                    result.history.push(DataItem { x, y: y.0 })
                },
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

    /// Returns a json string with min/max temperature values
    ///
    /// # Arguments
    ///
    /// * 'source' - sensor id (source)
    /// * 'from' - utc datetime in the rfc3339 format
    /// * 'to' - utc datetime in the rfc3339 format (non-inclusive)
    pub fn get_min_max(&self, source: &str, from: &str, to: &str) -> Result<String, DBError> {
        let start = DateTime::parse_from_rfc3339(from)?.with_timezone(&Utc);
        let end = DateTime::parse_from_rfc3339(to)?.with_timezone(&Utc);

        // Get min/max
        let mut stmt = self.db_conn.prepare(
            "SELECT MIN(temperature), MAX(temperature) 
                FROM weather
                WHERE source = ?1 AND datetime >= ?2 AND datetime < ?3;",
        )?;
        

        let mut result: Option<MinMax> = None;

        let rows = &mut stmt.query(params![source, start.timestamp(), end.timestamp()])?;
        if let Some(row) = rows.next()? {
            result = Some(MinMax {
                min: row.get(0).unwrap_or(0.0),
                max: row.get(1).unwrap_or(0.0),
            });
        }

        // Make sure that we have at least one data point in case there hasn't yet been any data recorded
        if result.is_none() {
            let mut stmt = self.db_conn.prepare(
                "SELECT temperature
                FROM weather
                WHERE source = ?1
                ORDER BY datetime DESC LIMIT 1;",
            )?;
            let response: rusqlite::Result<f64> = stmt.query_one(params![source], |row| row.get(0));
            match response {
                Ok(y) => {
                    result = Some(MinMax { min: y, max: y });
                },
                Err(e) => {
                    if e != rusqlite::Error::QueryReturnedNoRows {
                        return Err(DBError::from(e));
                    }
                }
            }
        }

        Ok(serde_json::to_string_pretty(&result.unwrap())?)
    }

    /// Returns the last recorded wind speed and humidity for the given datetime
    ///
    /// # Arguments
    ///
    /// * 'source' - sensor id (source)
    /// * 'date_time' - datetime to get data for
    pub fn get_wind_and_humidity(&self, source: &str, date_time: DateTime<Utc>) -> Result<Option<(f64, u8)>, DBError> {
        let start = date_time.timestamp() - 7200;
        let end = date_time.timestamp();

        let mut stmt = self.db_conn.prepare(
            "SELECT wind_speed, humidity
            FROM weather
            WHERE source = ?1 AND datetime <= ?2 AND datetime > ?3
            ORDER BY datetime desc LIMIT 1;",
        )?;

        let response: rusqlite::Result<(f64,u8)>  = stmt.query_one(params![source, end, start], |row| {
            Ok((row.get(0)?, row.get(1)?))
        });

        match response {
            Ok(r) => {
                Ok(Some(r))
            },
            Err(e) => {
                if e != rusqlite::Error::QueryReturnedNoRows {
                    Err(DBError::from(e))
                } else {
                    Ok(None)
                }
            }
        }
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