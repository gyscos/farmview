#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]
extern crate clap;
extern crate serde;
extern crate serde_json;
extern crate toml;
extern crate xdg;
extern crate ssh2;
extern crate rayon;

mod config;
mod fetch;

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
    println!("Using config file {:?}", config_path);

    // TODO: at least print an error message if config cannot be loaded
    let config = match config::read_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            println!("Error loading config: {:?}", e);
            config::Config::default()
        }
    };

    let port = config.http.as_ref().map_or(8080, |http| http.port);

    println!("Running webserver on port {:?}", port);

    println!("{:?}", fetch::fetch_data(&config));

}

fn default_config_path() -> std::io::Result<std::path::PathBuf> {
    xdg::BaseDirectories::new().unwrap().place_config_file("farmview.toml")
}
