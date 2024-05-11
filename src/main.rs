use std::{env, process, time};
use dive_reporter::Config;

fn main() {
    let timer = time::Instant::now();

    let args = env::args();
    let config = Config::build(args).unwrap_or_else(|err| {
        eprintln!("Error when parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = dive_reporter::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }

    println!("Elapsed: {:.2?}", timer.elapsed());
}
