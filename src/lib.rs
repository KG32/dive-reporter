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
            // None => return Err("Path missing"),
            None => "/Users/kubagroblewski/Documents/dive-reporter-uddf/kg-ss/165-167.uddf".to_string()
        };
        Ok(Config {
            path,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // let stats = Stats::from_dir(&config.path)?;
    let stats = Stats::from_file(&config.path)?;
    stats.print();
    Ok(())
}
