use std::error::Error;
use crate::parser::{self, UDDFDoc};

static DEV_FILE_PATH: &str = "/Users/kubagroblewski/Documents/dive-reporter-tmp/Perdix 2[A76240BD]#61_2023-10-28.uddf";

pub type Depth = f32;
pub type Seconds = usize;

pub struct Stats {
    total_no: usize,
    total_time: Seconds,
    depth_max: Depth,
}

#[derive(Debug)]
pub struct DiveStats {
    total_time: Seconds,
    depth_max: Depth,
    time_in_deco: Seconds,
}

impl DiveStats {
    pub fn new() -> DiveStats {
        DiveStats {
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

impl Stats {
    pub fn from_dir(path: &str) -> Result<Stats, Box<dyn Error>> {
        let test_file = parser::parse_file(DEV_FILE_PATH)?;
        let dive_stats = Self::get_dive_stats(test_file);
        println!("dive stats: {:?}", dive_stats);

        Ok(Stats {
            total_no: 0,
            total_time: 0,
            depth_max: 0.0,
        })
    }

    fn get_dive_stats(dive: UDDFDoc) -> Result<DiveStats, Box<dyn Error>> {
        dbg!("Get file stats");
        let mut dive_stats = DiveStats::new();
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
