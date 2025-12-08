/// Calculates the perceived temperature in Celsius.
///
/// # Arguments
///
/// * 'temp' - temperature in Celsius
/// * 'humidity' - humidity in percentage
/// * 'wind_speed' - wind speed in m/s
pub fn perceived_temperature(temp: f64, humidity: f64, wind_speed: f64) -> f64 {
    let temp = celsius_to_fahrenheit(temp);
    let mph = mps_to_mph(wind_speed);

    if temp <= 50.0 && mph > 3.0 {
        fahrenheit_to_celsius(wind_chill(temp, mph))
    } else {
        fahrenheit_to_celsius(heat_index(temp, humidity))
    }
}

/// Calculates the wind chill in Fahrenheit.
/// https://www.weather.gov/safety/cold-wind-chill-chart
///
/// # Arguments
///
/// * 'temp' - temperature in Fahrenheit
/// * 'wind_speed' - wind speed in miles per hour
fn wind_chill(temp: f64, wind_speed: f64) -> f64 {
    35.74 + 0.6215 * temp - 35.75 * wind_speed.powf(0.16) + 0.4275 * temp * wind_speed.powf(0.16)
}


/// Calculates the heat index in Fahrenheit.
/// The formula used is based on the original formula published by the National Oceanic and Atmospheric Administration (NOAA).
/// https://www.wpc.ncep.noaa.gov/html/heatindex_equation.shtml
///
/// # Arguments
///
/// * 'temp' - temperature in Fahrenheit
/// * 'humidity' - humidity in percentage
fn heat_index(temp: f64, humidity: f64) -> f64 {
    let mut heat_index = 0.5 * (temp + 61.0 + ((temp-68.0)*1.2) + (humidity * 0.094));

    if heat_index >= 80.0 {
        heat_index = -42.379 +
            2.04901523 * temp +
            10.14333127 * humidity -
            0.22475541 * temp * humidity -
            0.00683783 * temp * temp -
            0.05481717 * humidity * humidity +
            0.00122874 * temp * temp * humidity +
            0.00085282 * temp * humidity * humidity -
            0.00000199 * temp * temp * humidity * humidity;

        if humidity < 13.0 && temp >= 80.0 && temp <= 112.0 {
            heat_index -= ((13.0 - humidity) / 4.0) * ((17.0 - (temp - 95.0).abs()) / 17.0).sqrt();
        } else if humidity > 85.0 && temp >= 80.0 && temp <= 87.0 {
            heat_index +=  ((humidity - 85.0) / 10.0) * ((87.0 - temp) / 5.0);
        }
    }

    heat_index
}

/// Convert Celsius to Fahrenheit
///
/// # Arguments
///
/// * 'temp' - temperature in Celsius
fn celsius_to_fahrenheit(temp: f64) -> f64 {
    temp * 1.8 + 32.0
}

/// Converts a temperature value from Fahrenheit to Celsius and rounds the result to the nearest tenth.
///
/// # Arguments
///
/// * 'temp' - temperature in Fahrenheit
fn fahrenheit_to_celsius(temp: f64) -> f64 {
    ((temp - 32.0) / 1.8 * 10.0).round() / 10.0
}

/// Converts from meter per second to miles per hour.
///
/// # Arguments
///
/// * 'mps' - meter per second
fn mps_to_mph(mps: f64) -> f64 {
    mps * (1.0 / 1.609344 * 3.6)
}
