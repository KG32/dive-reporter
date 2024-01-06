#![allow(warnings)]

mod stats;
mod parser;

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
    Stats::from_dir(&config.path)?;

    Ok(())
}
