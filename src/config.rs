use std::fs;
use std::path::Path;
use toml;
use std::io::{self, Read, Write};

// Serialization made with serde

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(skip_serializing_if="Option::is_none")]
    pub http: Option<HttpConfig>,
    #[serde(skip_serializing_if="Option::is_none")]
    pub default: Option<AuthConfig>,

    pub hosts: Vec<HostConfig>,
    // Seconds between two refresh rates
    #[serde(skip_serializing_if="Option::is_none")]
    pub refresh_delay: Option<u64>,
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

pub fn read_config<P: AsRef<Path>>(filename: P) -> io::Result<Config> {
    let mut file = try!(fs::File::open(filename));
    let mut buffer = String::new();
    try!(file.read_to_string(&mut buffer));
    toml::decode_str(&buffer).ok_or(io::Error::new(io::ErrorKind::Other,
                                                   format!("Could not load \
                                                            toml.")))
}

pub fn write_config<P: AsRef<Path>>(filename: P,
                                    config: &Config)
                                    -> io::Result<()> {
    let buffer = toml::encode_str(config);
    let mut file = try!(fs::File::create(filename));
    try!(file.write_all(buffer.as_bytes()));

    Ok(())
}
