use postgres::{Connection, GenericConnection, TlsMode};
use toml::Value;

use std::fs::File;
use std::io::prelude::*;

pub fn with_db<F: Fn(&GenericConnection)>(f: F) {
    let mut file = File::open("Rocket.toml").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let value = contents.parse::<Value>().unwrap();
    let url = value["development"]["database_url"].as_str().unwrap();
    let conn = Connection::connect(url, TlsMode::None).unwrap();
    let tx = conn.transaction().unwrap();
    f(&tx);
}
