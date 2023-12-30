mod stats;
mod parser;

use stats::Stats;

// static DEV_FILE_PATH: &str = "/Users/kubagroblewski/Documents/dive-reporter-tmp/Perdix 2[A76240BD]#61_2023-10-28.uddf";

pub fn run() {
    let _ = match Stats::from_dir("test") {
        Ok(_) => { println!("done.")},
        Err(err) => {
            println!("Error: {}", err);
        }
    };
}
