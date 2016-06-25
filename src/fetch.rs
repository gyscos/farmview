use config;
use data::{Data, HostData};

use std::error;
use std::path;
use std::io::Write;
use std::net::TcpStream;

use rayon::prelude::*;
use serde_json;
use ssh2;


pub fn fetch_data(config: &config::Config) -> Data {
    let mut result = Vec::new();
    config.hosts
          .par_iter()
          .map(|host| fetch_host_data(host, config.default.as_ref()).ok())
          .collect_into(&mut result);

    // Post-process the data, using the configuration in parallel
    for (conf, host) in config.hosts.iter().zip(&mut result) {
        // Skip hosts without result
        if let &mut Some(ref mut host) = host {
            // Only keep disks that are not ignored
            host.disks.retain(|data| {
                !conf.ignored_disks.contains(&data.device) &&
                !conf.ignored_disks.contains(&data.mount)
            });
        }
    }

    Data { hosts: result }
}

fn authenticate(sess: &mut ssh2::Session,
                host: &config::HostConfig,
                default: Option<&config::AuthConfig>)
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

fn connect(host: &config::HostConfig,
           default: Option<&config::AuthConfig>)
           -> Result<(TcpStream, ssh2::Session), Box<error::Error + Send + Sync>> {

    // TODO: Don't panic on error
    let tcp = try!(TcpStream::connect((&*host.address, 22)));

    // An error here means something very wrong is going on.
    let mut sess = ssh2::Session::new().unwrap();
    try!(sess.handshake(&tcp));
    try!(authenticate(&mut sess, host, default));

    Ok((tcp, sess))
}


fn fetch_host_data(host: &config::HostConfig,
                   default: Option<&config::AuthConfig>)
                   -> Result<HostData, Box<error::Error + Send + Sync>> {

    // `tcp` needs to survive the scope, because on drop it closes the connection.
    let (_tcp, sess) = try!(connect(host, default));

    let mut channel = try!(sess.channel_session());
    try!(channel.exec(&format!("./fetch.py {}", host.iface)));
    // A JSON error here means the script went mad.
    // ... or just a connection issue maybe?
    // TODO: don't panic on error
    let result = try!(serde_json::from_reader(channel));

    Ok(result)
}

fn prepare_host(host: &config::HostConfig,
                default: Option<&config::AuthConfig>)
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

pub fn prepare_hosts(config: &config::Config)
                     -> Vec<Option<Box<error::Error + Send + Sync>>> {
    let mut result = Vec::new();
    config.hosts
          .par_iter()
          .map(|host| prepare_host(host, config.default.as_ref()).err())
          .collect_into(&mut result);
    result
}
