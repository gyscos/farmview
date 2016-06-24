use std::fs;
use std::path::Path;
use toml;
use std::io::{self, Read, Write};

// Serialization made with serde

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub http: Option<HttpConfig>,
    pub default: Option<AuthConfig>,

    pub hosts: Vec<HostConfig>,
    // Seconds between two refresh rates
    pub refresh_delay: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfig {
    pub port: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HostConfig {
    pub name: String,
    pub address: String,
    pub iface: String,
    pub ignored_disks: Vec<String>,
    pub auth: Option<AuthConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub login: String,
    pub keypair: Option<String>,
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
