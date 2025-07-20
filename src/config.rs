use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct InfluxDb {
    pub host: String,
    pub org: String,
    pub token: String,
    pub bucket: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub influxdb: InfluxDb,
    pub listen_port: u16,
}

impl Config {
    pub fn new<P>(path: P) -> anyhow::Result<Self>
    where
        P: Into<PathBuf>,
    {
        let path: PathBuf = path.into();
        let s = fs::read_to_string(path.clone()).context(format!("openening file {path:?}"))?;
        let cfg: Config = toml::from_str(s.as_str()).context("parse toml config")?;
        Ok(cfg)
    }
}
