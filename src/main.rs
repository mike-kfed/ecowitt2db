use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Form, State},
    routing::post,
};
use futures::prelude::*;
use influxdb2::Client;
use influxdb2::models::DataPoint;
use serde::Deserialize;

mod config;

/// Incoming form data from the Ecowitt compatible device.
/// Field names must match weather stations output.
#[derive(Deserialize, Debug)]
#[allow(dead_code, reason = "some station data is static")]
struct EcowittPayload {
    // Station data
    freq: String,
    #[serde(rename = "PASSKEY")]
    passkey: String, // The device's MD5 hashed passkey
    stationtype: String,
    model: String,
    runtime: u64,
    wh65batt: u8, // battery status (apparently, 0 = OK, 1 = low)
    heap: i32,    // TODO figure out?
    dateutc: String,
    interval: u32,

    // Weather data
    tempf: f64,        // Outdoor Temperature (Fahrenheit)
    humidity: i32,     // Outdoor Humidity (%)
    tempinf: f64,      // Indoor Temperature (Fahrenheit)
    humidityin: i32,   // Indoor Humidity (%)
    windspeedmph: f64, // Wind Speed (miles/h)
    windgustmph: f64,
    winddir: u16,        // Wind Direction (0-360°)
    baromabsin: f64,     // Absolute Pressure (inHg)
    baromrelin: f64,     // Relative Pressure (inHg)
    solarradiation: f64, // Solar Radiation (W/m²)
    uv: i32,             // UV Index
    vpd: f64,            // vapor pressure deficit (kPa)

    rainratein: f64, // Rain Rate (in/hr)
    eventrainin: f64,
    totalrainin: f64,
    hourlyrainin: f64,
    dailyrainin: f64, // Daily Rain (in)
    weeklyrainin: f64,
    monthlyrainin: f64,
    yearlyrainin: f64,
}

#[derive(Debug)]
#[allow(dead_code, reason = "absolute pressure is not meaningful")]
struct MetricWeather {
    temp_outdoors: f64,
    humidity_outdoors: i32,
    temp_indoors: f64,
    humidity_indoors: i32,
    windspeed: f64,
    windgust: f64,
    winddir: u16,
    pressure_abs: f64, // inches of mercury
    pressure_rel: f64,
    rain: Rain,
    solarradiation: f64,
    uv: i32,
    vpd: f64,
}

#[derive(Debug)]
#[allow(dead_code, reason = "aggregate rain info can be computed")]
struct Rain {
    rainrate: f64,
    eventrain: f64,
    totalrain: f64,
    hourlyrain: f64,
    dailyrain: f64,
    weeklyrain: f64,
    monthlyrain: f64,
    yearlyrain: f64,
}

impl From<EcowittPayload> for MetricWeather {
    fn from(e: EcowittPayload) -> Self {
        let rain = Rain {
            rainrate: e.rainratein * 25.4, // inch to mm
            dailyrain: e.dailyrainin * 25.4,
            eventrain: e.eventrainin * 25.4,
            totalrain: e.totalrainin * 25.4,
            hourlyrain: e.hourlyrainin * 25.4,
            weeklyrain: e.weeklyrainin * 25.4,
            monthlyrain: e.monthlyrainin * 25.4,
            yearlyrain: e.yearlyrainin * 25.4,
        };
        Self {
            temp_outdoors: fahrenheit_to_celsius(e.tempf),
            humidity_outdoors: e.humidity,
            temp_indoors: fahrenheit_to_celsius(e.tempinf),
            humidity_indoors: e.humidityin,
            windspeed: e.windspeedmph * 1.60934, // mph to kph
            windgust: e.windgustmph * 1.60934,   // mph to kph
            winddir: e.winddir,
            rain,
            pressure_abs: e.baromabsin * 33.863889532611, // inches of mercury to hPa
            pressure_rel: e.baromrelin * 33.863889532611, // inches of mercury to hPa
            solarradiation: e.solarradiation,             // W/m^3
            uv: e.uv,
            vpd: e.vpd,
        }
    }
}

fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

async fn handle_ecowitt_data(
    State(cfg): State<Arc<config::Config>>,
    Form(payload): Form<EcowittPayload>,
) -> String {
    let battery_ok = payload.wh65batt == 0;
    let data: MetricWeather = payload.into();

    let bucket = cfg.influxdb.bucket.as_str();
    let client = Client::new(
        cfg.influxdb.host.as_str(),
        cfg.influxdb.org.as_str(),
        cfg.influxdb.token.as_str(),
    );

    let points = [
        DataPoint::builder("outdoor")
            .field("temperature", data.temp_outdoors)
            .field("humidity", i64::from(data.humidity_outdoors))
            .field("windspeed", data.windspeed)
            .field("windgust", data.windgust)
            .field("winddir", i64::from(data.winddir))
            .field("rainrate", data.rain.rainrate)
            .field("pressure", data.pressure_rel)
            .field("solarradiation", data.solarradiation)
            .field("uv", i64::from(data.uv))
            .field("vpd", data.vpd)
            .build()
            .unwrap(),
        DataPoint::builder("indoor")
            .field("temperature", data.temp_indoors)
            .field("humidity", i64::from(data.humidity_indoors))
            .build()
            .unwrap(),
        DataPoint::builder("station")
            .field("battery_ok", battery_ok)
            .build()
            .unwrap(),
    ];

    match client.write(bucket, stream::iter(points)).await {
        Ok(_) => "OK".to_string(),
        Err(e) => {
            eprintln!("{e:?}");
            "ERROR".to_string()
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(config::Config::new("ecowitt2db.toml")?);

    let app = Router::new()
        .route("/data/report", post(handle_ecowitt_data))
        .route("/data/report/", post(handle_ecowitt_data))
        .with_state(config.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.listen_port));
    println!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
