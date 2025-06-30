pub mod errors;
mod models;

use std::ops::Add;
use chrono::{DateTime, DurationRound, Local, TimeDelta, Utc};
use log::error;
use rusqlite::{params, Connection};
use crate::manager_db::errors::DBError;
use crate::manager_db::models::{DataItem, TwoDaysMinMax};

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
                constraint primary_key primary key (source, datetime)
           )",
           [],
        )?;
        
        Ok(DB { db_conn, max_age_in_days })
    }
    
    /// Inserts a record in the database
    /// 
    /// # Arguments
    ///
    /// * 'source' - sensor id (source)    
    /// * 'temp' - temperature
    /// * 'humidity' - humidity
    pub fn insert_record(&self, source: &str, temp: f64, humidity: u8) -> Result<(), DBError> {
        
        self.db_conn.execute(
            "INSERT INTO weather (source, datetime, temperature, humidity) values (?1, ?2, ?3, ?4)",
            params![source, Utc::now().timestamp(), temp, humidity],
        )?;
        
        Ok(())
    }
    
    /// Returns a json string with whatever temperatures are recorded between given boundaries
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
    /// * 'from' - local datetime in the rfc3339 format
    /// * 'to' - local datetime in the rfc3339 format
    pub fn get_temp_history(&self, source: &str, from: &str, to: &str) -> Result<String, DBError> {
        let from_datetime = DateTime::parse_from_rfc3339(from)?.with_timezone(&Local);
        let from_timestamp = DateTime::parse_from_rfc3339(from)?.with_timezone(&Utc).timestamp();
        let to_timestamp = DateTime::parse_from_rfc3339(to)?.with_timezone(&Utc).timestamp();
        
        let mut result: Vec<DataItem<f64>> = Vec::new();
        
        // Get what may naturally be between the given time boundary from the database
        let mut stmt = self.db_conn.prepare(
            "SELECT datetime, temperature 
                FROM weather
                WHERE source = ?1 AND datetime BETWEEN ?2 AND ?3
                ORDER BY datetime;",
        )?;
        let mut rows = stmt.query(params![source, from_timestamp, to_timestamp])?;

        while let Some(row) = rows.next()? {
            let timestamp: i64 = row.get(0)?;
            let x = DateTime::from_timestamp(timestamp, 0).unwrap().with_timezone(&Local);
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
    
    /// Returns a json string with two days min/max temperature values
    ///
    /// # Arguments
    ///
    /// * 'source' - sensor id (source)
    pub fn get_two_day_min_max(&self, source: &str) -> Result<String, DBError> {
        let today_start = Local::now().duration_trunc(TimeDelta::days(1)).unwrap();
        let today_end = today_start.add(TimeDelta::seconds(86399));
        let yesterday_start = today_start.add(TimeDelta::days(-1));
        let yesterday_end = today_end.add(TimeDelta::days(-1));

        // Get min/max
        let mut stmt = self.db_conn.prepare(
            "SELECT MIN(temperature), MAX(temperature) 
                FROM weather
                WHERE source = ?1 AND datetime BETWEEN ?2 AND ?3;",
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
                let trunc_time = Local::now().add(TimeDelta::days(-1 * self.max_age_in_days)).with_timezone(&Utc).timestamp();
                if let Err(e) = stmt.query(params![trunc_time]) {
                    error!("error while deleting rows: {}", e);
                }
            },
            Err(e) => { error!("error while preparing delete statement: {}", e); }
        }
    }
}