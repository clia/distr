//! Server config

use anyhow::Result;
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};

/// Basic config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Base {
    /// Running Enviroment: dev, test, prod
    pub env: String,
}

/// HTTP config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Http {
    /// Outer HTTP port
    pub outer_port: usize,
    /// Outer HTTPS port
    pub outer_ssl_port: usize,
    /// Inner HTTP port
    pub inner_port: usize,
    /// Inner HTTPS port
    pub inner_ssl_port: usize,
}

/// Site proxy config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SiteProxy {
    /// config proxy host, with schema and port
    pub host: String,
    /// exclude some path, use local file
    pub exclude: Option<Vec<String>>,
}

/// Site config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Site {
    /// comma/space seperated multi host config
    pub hosts: String,
    /// site root dir
    pub root: String,
    /// site proxy config
    pub proxy: Option<SiteProxy>,
}

/// Server config
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub base: Base,
    pub http: Http,
    pub sites: Vec<Site>,
}

impl ServerConfig {
    pub fn load_config() -> Result<Self> {
        let builder = Config::builder()
            .set_default("base.env", "dev")?
            .set_default("http.outer_port", 80)?
            .set_default("http.outer_ssl_port", 443)?
            .set_default("http.inner_port", 20080)?
            .set_default("http.inner_ssl_port", 20443)?
            .add_source(File::new("conf/config", FileFormat::Json5))
            .set_override("override", "1")?;

        match builder.build() {
            Ok(config) => config.try_into().map_err(|e| e.into()),
            Err(err) => Err(err.into()),
        }
    }
}
