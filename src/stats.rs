use std::{error::Error, fs, path::PathBuf, fmt::format};
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
    total_time: Seconds,
    depth_max: Depth,
    time_in_deco: Seconds,
    deco_dives_no: usize,
}

impl Stats {
    pub fn from_dir(path: &str) -> Result<Stats, Box<dyn Error>> {
        println!("\nDives folder: {}", path);
        let mut stats = Stats {
            dives_no: 0,
            total_time: 0,
            depth_max: 0.0,
            deco_dives_no: 0,
            time_in_deco: 0,
        };

        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            if (Self::validate_uddf_target(&entry_path)) {
                let dive_stats = Self::handle_file(entry_path.to_str().unwrap())?;
                stats.update_with_dive_data(dive_stats);
            }
        }

        Ok(stats)
    }

    fn update_with_dive_data(&mut self, dive_stats: Dive) {
        self.dives_no += 1;
        self.total_time += dive_stats.total_time;
        if dive_stats.depth_max > self.depth_max {
            self.depth_max = dive_stats.depth_max;
        }
        if dive_stats.time_in_deco > 0 {
            self.time_in_deco += dive_stats.time_in_deco;
            self.deco_dives_no += 1;
        }
    }

    fn handle_file(path: &str) -> Result<Dive, Box<dyn Error>> {
        println!("Analyzing dive: {}", path);
        let test_file = parser::parse_file(path)?;
        let dive_stats = Self::get_dive_stats(test_file)?;
        Ok(dive_stats)
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

    pub fn print(&self) -> () {
        println!("\n---------- STATS ----------");
        println!("Dives: {}", self.dives_no);
        println!("Total time: {}", Self::seconds_to_readable(self.total_time));
        println!("Max depth: {}m", self.depth_max);
        println!("Deco dives: {}", self.deco_dives_no);
        println!("Total time in deco: {}", Self::seconds_to_readable(self.time_in_deco));
    }

    fn seconds_to_readable(s: usize) -> String {
        let seconds = s % 60;
        let minutes = (s / 60) % 60;
        let hours = (s / 60) / 60;
        format!("{hours}h {minutes}m {seconds}s")
    }
}
