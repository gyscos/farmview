use config::{Config, HostConfig, AuthConfig, LocationConfig};
use data::{Data, HostData};
use ips::IpBlock;

use std::error;
use std::path;
use std::io::Write;
use std::net::TcpStream;

use rayon::prelude::*;
use serde_json;
use ssh2;
use time;

pub fn fetch_data(config: &Config) -> Data {
    let mut result: Vec<HostData> = config.hosts
        .par_iter()
        .filter_map(|host| {
            match fetch_host_data(host,
                                  config.default.as_ref(),
                                  &config.locations) {
                Ok(mut result) => {
                    result.disks.retain(|data| {
                        host.ignored_disks
                            .as_ref()
                            .map(|disks| {
                                !disks.contains(&data.device) &&
                                    !disks.contains(&data.mount)
                            })
                        .unwrap_or(true)
                    });
                    Some(result)
                },
                Err(err) => {
                    println!("Error fetching {}: {:?}", host.address, err);
                    None
                }
            }
        })
        .collect();

    let empty = String::new();
    result.sort_by(|a, b| {
        a.location
            .as_ref()
            .unwrap_or(&empty)
            .cmp(b.location.as_ref().unwrap_or(&empty))
    });

    let now = time::now().to_timespec().sec;
    Data {
        hosts: result,
        update_time: now,
    }
}

fn authenticate(sess: &mut ssh2::Session,
                host: &HostConfig,
                default: Option<&AuthConfig>)
                -> Result<(), ssh2::Error> {

    // Do we have an authentication? Or do we have a default one?
    if let Some(auth) = host.auth.as_ref().or(default) {
        if let Some(ref password) = auth.password {
            // Maybe we log in with a password?
            try!(sess.userauth_password(&auth.login, password));
        } else if let Some(ref keypair) = auth.keypair {
            // Or maybe with an identity file?
            try!(sess.userauth_pubkey_file(&auth.login,
                                           None,
                                           path::Path::new(keypair),
                                           None));
        }
    }
    Ok(())
}

type BoxedError = Box<error::Error + Send + Sync>;

fn connect(host: &HostConfig,
           default: Option<&AuthConfig>)
           -> Result<(TcpStream, ssh2::Session), BoxedError> {

    // TODO: Don't panic on error
    let tcp = try!(TcpStream::connect((&*host.address, 22)));

    // An error here means something very wrong is going on.
    let mut sess = ssh2::Session::new().unwrap();
    try!(sess.handshake(&tcp));
    try!(authenticate(&mut sess, host, default));

    Ok((tcp, sess))
}


fn fetch_host_data(host: &HostConfig,
                   default: Option<&AuthConfig>,
                   locations: &[LocationConfig])
                   -> Result<HostData, Box<error::Error + Send + Sync>> {

    // `tcp` needs to survive the scope,
    // because on drop it closes the connection.
    // But we're not using it, so an underscore
    // will avoid `unused` warnings.
    let (_tcp, sess) = try!(connect(host, default));

    let mut channel = try!(sess.channel_session());
    try!(channel.exec(&format!("./fetch.py {}", host.iface)));
    // A JSON error here means the script went mad.
    // ... or just a connection issue maybe?
    let mut result: HostData = try!(serde_json::from_reader(channel));
    let location = result.network
        .as_ref()
        .and_then(|n| n.ip.as_ref())
        .and_then(|ip| find_location(ip, locations));

    result.location = location.or_else(|| host.location.clone());

    Ok(result)
}

fn find_location(ip: &str, locations: &[LocationConfig]) -> Option<String> {
    locations.iter().find(|l| match_ip(ip, &l.ips)).map(|l| l.name.clone())
}

fn match_ip(ip: &str, mask: &str) -> bool {
    IpBlock::new(mask).matches(ip)
}

fn prepare_host(host: &HostConfig,
                default: Option<&AuthConfig>)
                -> Result<(), Box<error::Error + Send + Sync>> {

    // Directly include the script in the executable
    let script_data = include_str!("../data/fetch.py");

    // `tcp` needs to survive the scope, because on drop it closes the connection.
    let (_tcp, sess) = try!(connect(host, default));
    let mut remote_file = try!(sess.scp_send(path::Path::new("fetch.py"),
                                             0o755,
                                             script_data.len() as u64,
                                             None));
    try!(remote_file.write_all(script_data.as_bytes()));
    Ok(())
}

pub fn prepare_hosts(config: &Config)
                     -> Vec<Option<Box<error::Error + Send + Sync>>> {
    let mut result = Vec::new();
    config.hosts
        .par_iter()
        .map(|host| prepare_host(host, config.default.as_ref()).err())
        .collect_into(&mut result);
    result
}
