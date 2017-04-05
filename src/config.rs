use std::fs;
use std::path::Path;
use toml;
use std::io::{Read, Write};
use errors::*;

// Serialization made with serde

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(skip_serializing_if="Option::is_none")]
    pub http: Option<HttpConfig>,

    pub locations: Vec<LocationConfig>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub default: Option<AuthConfig>,

    pub hosts: Vec<HostConfig>,
    // Seconds between two refresh rates
    #[serde(skip_serializing_if="Option::is_none")]
    pub refresh_delay: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct LocationConfig {
    pub name: String,
    pub ips: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HttpConfig {
    pub port: u16,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HostConfig {
    pub name: String,
    pub address: String,
    pub iface: String,

    #[serde(skip_serializing_if="Option::is_none")]
    pub ignored_disks: Option<Vec<String>>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub auth: Option<AuthConfig>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub location: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AuthConfig {
    pub login: String,
    #[serde(skip_serializing_if="Option::is_none")]
    pub keypair: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub password: Option<String>,
}

pub fn read_config<P: AsRef<Path>>(filename: P) -> Result<Config> {
    let mut file = fs::File::open(filename).chain_err(|| "could not open config file")?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).chain_err(|| "could not read config")?;
    toml::de::from_str(&buffer).chain_err(|| "could not parse config")
}

pub fn write_config<P: AsRef<Path>>(filename: P,
                                    config: &Config)
                                    -> Result<()> {
    let buffer = toml::ser::to_vec(config).chain_err(|| "could not serialize config")?;
    let mut file = fs::File::create(filename).chain_err(|| "could not create config file")?;
    file.write_all(&buffer).chain_err(|| "could not write config")?;

    Ok(())
}
