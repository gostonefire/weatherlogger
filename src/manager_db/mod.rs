pub mod errors;
mod models;

use chrono::{DateTime, Local, Utc};
use rusqlite::{params, Connection};
use crate::manager_db::errors::DBError;
use crate::manager_db::models::DataItem;

pub struct DB {
    db_conn: Connection,
    
}

impl DB {
    
    /// Creates a new instance of DB
    /// 
    /// # Arguments
    /// 
    /// * 'db_path' - full path to db file
    pub fn new(db_path: &str) -> Result<Self, DBError> {
        let db_conn = Connection::open(db_path)?;
        db_conn.execute(
            "CREATE TABLE IF NOT EXISTS temp_hum (
                datetime integer primary key,
                temperature real null,
                humidity integer null
         )",
            [],
        )?;
        
        Ok(DB { db_conn })
    }
    
    /// Inserts a record in the database
    /// 
    /// # Arguments
    /// 
    /// * 'temp' - temperature
    /// * 'humidity' - humidity
    pub fn insert_record(&self, temp: f64, humidity: u8) -> Result<(), DBError> {
        
        self.db_conn.execute(
            "INSERT INTO temp_hum (datetime, temperature, humidity) values (?1, ?2, ?3)",
            params![Utc::now().timestamp(), temp, humidity],
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
    /// * 'from' - local datetime in the rfc3339 format
    /// * 'to' - local datetime in the rfc3339 format
    pub fn get_temp_history(&self, from: &str, to: &str) -> Result<String, DBError> {
        let from_datetime = DateTime::parse_from_rfc3339(from)?.with_timezone(&Local);
        let from_timestamp = DateTime::parse_from_rfc3339(from)?.with_timezone(&Utc).timestamp();
        let to_timestamp = DateTime::parse_from_rfc3339(to)?.with_timezone(&Utc).timestamp();
        
        let mut result: Vec<DataItem<f64>> = Vec::new();
        
        // Get what may naturally be between the given time boundary from the database
        let mut stmt = self.db_conn.prepare(
            "SELECT datetime, temperature 
                FROM temp_hum
                WHERE datetime BETWEEN ?1 AND ?2
                ORDER BY datetime;",
        )?;
        let mut rows = stmt.query(params![from_timestamp, to_timestamp])?;

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
                FROM temp_hum
                ORDER BY datetime DESC LIMIT 1;",
            )?;
            let x =  from_datetime;
            let response: rusqlite::Result<f64> = stmt.query_one([], |row| row.get(0));
            match response {
                Ok(y) => result.insert(0, DataItem { x, y }),
                Err(e) => {
                    if e != rusqlite::Error::QueryReturnedNoRows {
                        return Err(DBError::from(e));
                    }
                }
            }
        }
        
        Ok(serde_json::to_string_pretty(&result)?)
    }
}