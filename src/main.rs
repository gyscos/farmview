#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tera;

#[macro_use]
extern crate error_chain;

extern crate clap;
extern crate serde_json;
extern crate toml;
extern crate xdg;
extern crate ssh2;
extern crate rayon;
extern crate crossbeam;
extern crate hyper;
extern crate reroute;
extern crate time;

mod errors {
    error_chain! {}
}

mod config;
mod data;
mod fetch;
mod server;
mod http;
mod ips;
mod slack;

use clap::{Arg, App};

fn read_args<'a>() -> clap::ArgMatches<'a> {
    App::new("farmview")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Alexandre Bury <alexandre.bury@gmail.com>")
        .about("A dashboard view for a server farm")
        .arg(Arg::with_name("CONFIG")
                 .help("Config file to use (defaults to ~/.config/farmview)")
                 .short("c")
                 .long("config")
                 .takes_value(true))
        .get_matches()
}

fn main() {
    let matches = read_args();

    let config_path = match matches.value_of("CONFIG") {
        Some(path) => std::path::PathBuf::from(path),
        None => default_config_path().unwrap(),
    };

    println!("Farmview v{}", env!("CARGO_PKG_VERSION"));

    println!("Using config file {:?}", config_path);
    let config = match config::read_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Error loading config: {:?}", e);
            config::Config::default()
        }
    };


    // And off we go!
    http::serve(config, move |config| {
        config::write_config(&config_path, config).unwrap();
    });
}

fn default_config_path() -> std::io::Result<std::path::PathBuf> {
    xdg::BaseDirectories::new().unwrap().place_config_file("farmview.toml")
}
