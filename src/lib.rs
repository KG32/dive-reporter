#![allow(warnings)]

mod common;
mod parser;
mod dive;
mod stats;

use std::error::Error;

use stats::Stats;

pub struct Config {
    pub path: String,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();
        let path = match args.next() {
            Some(arg) => arg,
            None => return Err("Path missing"),
        };
        Ok(Config {
            path,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let stats = Stats::new().from_path(&config.path)?;
    stats.print();
    Ok(())
}
