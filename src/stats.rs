use std::{error::Error, fs, path::PathBuf};
use crate::parser::{self, UDDFDoc};

static DEV_FILE_PATH: &str = "/Users/kubagroblewski/Documents/dive-reporter-tmp/Perdix 2[A76240BD]#61_2023-10-28.uddf";

pub type Depth = f32;
pub type Seconds = usize;

#[derive(Debug)]
pub struct Dive {
    total_time: Seconds,
    depth_max: Depth,
    time_in_deco: Seconds,
}

impl Dive {
    pub fn new() -> Dive {
        Dive {
            total_time: 0,
            depth_max: 0.0,
            time_in_deco: 0,
        }
    }

    fn update_depth_max(&mut self, depth: Depth) {
        self.depth_max = depth;
    }

    fn update_total_time(&mut self, time: Seconds) {
        self.total_time += time;
    }

    fn update_time_in_deco(&mut self, time: Seconds) {
        self.time_in_deco += time;
    }
}

pub struct Stats {
    total_no: usize,
    total_time: Seconds,
    depth_max: Depth,
    time_in_deco: Seconds,
}

impl Stats {
    pub fn from_dir(path: &str) -> Result<Stats, Box<dyn Error>> {
        println!("\nDives folder: {}", path);
        let stats = Stats {
            total_no: 0,
            total_time: 0,
            depth_max: 0.0,
            time_in_deco: 0,
        };

        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            if (Self::validate_uddf_target(&entry_path)) {
                let dive_stats = Self::from_file(entry_path.to_str().unwrap())?;
            } else {
                // todo
                // println!("Skipping {} - not a UDDF file", path.to_str().unwrap());
            }
        }

        Ok(Stats {
            total_no: 0,
            total_time: 0,
            depth_max: 0.0,
            time_in_deco: 0,
        })
    }

    pub fn from_file(path: &str) -> Result<Dive, Box<dyn Error>> {
        println!("\nAnalyzing dive: {}", path);
        let test_file = parser::parse_file(path)?;
        let dive_stats = Self::get_dive_stats(test_file)?;
        println!("\nDive stats: {:?}", dive_stats);
        Ok(dive_stats)
    }

    fn update_with_dive_stats(&mut self, dive: Dive) -> () {

    }

    fn validate_uddf_target(path: &PathBuf) -> bool {
        if !path.is_file() {
            return false;
        }
        let extension = path.extension();
        if extension.is_none() {
            return false;
        }
        let extension = extension.unwrap().to_str().unwrap();
        ["uddf", "UDDF"].contains(&extension)
    }

    fn get_dive_stats(dive: UDDFDoc) -> Result<Dive, Box<dyn Error>> {
        let mut dive_stats = Dive::new();
        let dive_data_points = dive
            .profile_data
            .repetition_group
            .dive
            .samples
            .waypoints;

        let mut last_waypoint_time: usize = 0;
        for data_point in dive_data_points {
            // max depth
            if data_point.depth > dive_stats.depth_max {
                dive_stats.update_depth_max(data_point.depth);
            }
            // time
            dive_stats.update_total_time(data_point.dive_time - last_waypoint_time);
            // decostop
            match data_point.decostops {
                Some(decostops) => {
                    let mandatory_stop_index = decostops
                        .iter()
                        .position(|ds| ds.kind == "mandatory");
                    if let Some(_) = mandatory_stop_index {
                        dive_stats.update_time_in_deco(data_point.dive_time - last_waypoint_time);
                    }
                },
                None => (),
            }

            // update last waypoint time
            last_waypoint_time = data_point.dive_time;
        }

        Ok(dive_stats)
    }
}
