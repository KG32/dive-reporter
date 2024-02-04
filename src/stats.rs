use std::{error::Error, fs, path::PathBuf, fmt::format};
use crate::parser::{self, UDDFDoc, Mix, DiveElem, WaypointElem};
use buehlmann_deco::{step, zhl16c};
use buehlmann_deco::model::ZHLModel;
use buehlmann_deco::gas::Gas;
use buehlmann_deco::step::Step;

pub type Depth = f64;
pub type Seconds = usize;

#[derive(Debug)]
pub struct DiveMetadata {
    last_gas: Gas,
}

#[derive(Debug)]
pub struct Dive {
    total_time: Seconds,
    depth_max: Depth,
    time_in_deco: Seconds,
    metadata: DiveMetadata,
}

impl Dive {
    pub fn new() -> Dive {
        Dive {
            total_time: 0,
            depth_max: 0.0,
            time_in_deco: 0,
            metadata: DiveMetadata { last_gas: Gas::new(0.21) }
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

pub struct UDDFData {
    gas_mixes: Option<Vec<Mix>>,
    dives_data: Vec<DiveElem>,
}

impl Stats {
    pub fn from_file(path: &str) -> Result<Stats, Box<dyn Error>> {
        println!("\nFile: {}", path);
        let mut stats = Stats {
            dives_no: 0,
            total_time: 0,
            depth_max: 0.0,
            deco_dives_no: 0,
            time_in_deco: 0,
        };
        let UDDFData { dives_data, gas_mixes } = stats.extract_data_from_file(path)?;
        for dive_data in dives_data {
            let dive_stats = stats.calc_dive_stats(dive_data)?;
            stats.update_with_dive_data(dive_stats);
        }

        Ok(stats)
    }

    fn extract_data_from_file(&self, path: &str) -> Result<UDDFData, Box<dyn Error>> {
        println!("Extracting dives from UDDF");
        let file = parser::parse_file(path)?;

        let gas_definitions = file.gas_definitions;
        let mut dives: Vec<DiveElem> = vec![];
        let repetition_groups = file
            .profile_data
            .repetition_group;
        for mut group in repetition_groups {
            dives.append(&mut group.dives);
        }

        Ok(UDDFData {
            gas_mixes: gas_definitions.gas_mixes,
            dives_data: dives,
        })
    }

    fn calc_dive_stats(&self, dive_elem: DiveElem) -> Result<Dive, Box<dyn Error>> {
        let mut dive = Dive::new();
        let mut deco = zhl16c();
        let dive_data_points = dive_elem.samples.waypoints;
        let mut last_waypoint_time: usize = 0;
        for data_point in dive_data_points {
            self.process_data_point(
                &mut dive,
                &mut deco,
                &data_point,
                &last_waypoint_time,
            );
            // update last waypoint time
            last_waypoint_time = data_point.dive_time;
        }

        Ok(dive)
    }

    fn process_data_point(&self, dive: &mut Dive, model: &mut ZHLModel, data_point: &WaypointElem, last_waypoint_time: &usize) -> () {
        // max depth
        if data_point.depth > dive.depth_max {
            dive.update_depth_max(data_point.depth);
        }

        // time
        let step_time = data_point.dive_time - last_waypoint_time;
        dive.update_total_time(step_time);

        // deco model
        // @todo real gas used
        let gas = &dive.metadata.last_gas;
        model.step(&data_point.depth, &step_time, gas);

        // gradient factors
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
