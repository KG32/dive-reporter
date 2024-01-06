use std::{error::Error, fs, path::PathBuf};
use crate::parser::{self, UDDFDoc};

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

#[derive(Debug)]
pub struct Stats {
    dives_no: usize,
    time: Seconds,
    depth_max: Depth,
    deco_time: Seconds,
    deco_dives_no: usize,
}

impl Stats {
    pub fn from_dir(path: &str) -> Result<Stats, Box<dyn Error>> {
        println!("\nDives folder: {}", path);
        let mut stats = Stats {
            dives_no: 0,
            time: 0,
            depth_max: 0.0,
            deco_time: 0,
            deco_dives_no: 0,
        };

        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            if (Self::validate_uddf_target(&entry_path)) {
                let dive_stats = Self::handle_file(entry_path.to_str().unwrap())?;
                stats.dives_no += 1;
                stats.time += dive_stats.total_time;
                if dive_stats.depth_max > stats.depth_max {
                    stats.depth_max = dive_stats.depth_max;
                }
                if dive_stats.time_in_deco > 0 {
                    stats.deco_time += dive_stats.time_in_deco;
                    stats.deco_dives_no += 1;
                }
            }
        }

        Ok(stats)
    }

    fn handle_file(path: &str) -> Result<Dive, Box<dyn Error>> {
        println!("Analyzing dive: {}", path);
        let test_file = parser::parse_file(path)?;
        let dive_stats = Self::get_dive_stats(test_file)?;
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
