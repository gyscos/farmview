use std::fs;
use std::path::Path;
use serde_json;

// Serialization made with serde

pub fn read_config<P: AsRef<Path>>(filename: P) -> serde_json::Result<Config> {
    let file = try!(fs::File::open(filename));
    serde_json::from_reader(file)
}

pub fn write_config<P: AsRef<Path>>(filename: P, config: &Config) -> serde_json::Result<()> {
    let mut file = try!(fs::File::create(filename));
    serde_json::to_writer(&mut file, config)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpConfig {
    pub port: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub login: String,
    pub keypair: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HostConfig {
    pub name: String,
    pub address: String,
    pub disks: Vec<String>,
    pub auth: Option<AuthConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub http: Option<HttpConfig>,
    pub default: Option<AuthConfig>,

    pub hosts: Vec<HostConfig>,
}
