use config;

use std::path;
use std::net::TcpStream;

use serde_json;
use ssh2;

#[derive(Default, Debug, Deserialize)]
pub struct Data {
    hostname: Option<String>,
    nproc: Option<u8>,
    uptime: Option<[f32; 3]>,
    memory: Option<MemoryData>,
    disks: Vec<DiskData>,
    network: Option<NetworkData>,
}

#[derive(Default, Debug, Deserialize)]
pub struct MemoryData {
    total: usize,
    used: usize,
}

#[derive(Default, Debug, Deserialize)]
pub struct DiskData {
    size: usize,
    available: usize,
    mount: String,
    device: String,
}

#[derive(Default, Debug, Deserialize)]
pub struct NetworkData {
    rx: f32,
    tx: f32,
}

pub fn fetch_data(config: &config::Config) -> Vec<Data> {
    config.hosts.iter().map(|host| fetch_host_data(host, config.default.as_ref()).unwrap()).collect()
}

fn fetch_host_data(host: &config::HostConfig, default: Option<&config::AuthConfig>) -> Result<Data, ssh2::Error> {
    let tcp = TcpStream::connect((&*host.address, 22)).unwrap();

    let mut sess = ssh2::Session::new().unwrap();
    try!(sess.handshake(&tcp));

    // Do we have an authentication? Or do we have a default one?
    if let Some(auth) = host.auth.as_ref().or(default) {
        if let Some(ref password) = auth.password {
            // Maybe we log in with a password?
            try!(sess.userauth_password(&auth.login, password));
        } else if let Some(ref keypair) = auth.keypair {
            // Or maybe with an identity file?
            try!(sess.userauth_pubkey_file(&auth.login, None, path::Path::new(keypair), None));
        }
    }

    let mut channel = try!(sess.channel_session());
    try!(channel.exec("./fetch.py"));
    Ok(serde_json::from_reader(channel).unwrap())
}
