use crate::common::{Depth, Seconds, GF};
use crate::dive::{Dive, DiveConfig};
use crate::parser::{self, DiveElem, Mix, UDDFDoc, WaypointElem};
use colored::*;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::{error::Error, fmt::format, fs, path::PathBuf};

#[derive(Clone, Debug, Default)]
pub struct StatsData {
    pub dives_no: usize,
    pub total_time: Seconds,
    pub depth_max: Depth,
    pub time_in_deco: Seconds,
    pub deco_dives_no: usize,
    pub gf_surf_max: GF,
    pub gf_99_max: GF,
    pub gf_end_max: GF,
    pub time_below: TimeBelowDepthData,
}

#[derive(Clone, Debug)]
pub struct Stats {
    pub stats_data: Arc<Mutex<StatsData>>,
}

pub type StatsOutput = Vec<(String, String)>;

pub type TimeBelowDepthData = Vec<(Depth, Seconds)>;

pub type GasMixesData = Option<Vec<Mix>>;

pub struct UDDFData {
    gas_mixes: GasMixesData,
    dives_data: Vec<DiveElem>,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            stats_data: Arc::new(Mutex::new(StatsData::default())),
        }
    }

    pub fn from_path(&self, path: &str) -> Result<Self, Box<dyn Error>> {
        let mut stats = Self::new();
        let path_meta = fs::metadata(path)?;
        if path_meta.is_file() {
            println!("File: {}", path);
            stats.from_file(path)?;
        } else if path_meta.is_dir() {
            println!("Directory: {}", path);
            stats.from_dir(path)?;
        } else {
            return Err("Unable to resolve file or directory".into());
        }
        stats.print_to_console();
        Ok(stats)
    }

    fn from_file(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let UDDFData {
            dives_data,
            gas_mixes,
        } = self.extract_data_from_file(path)?;
        dives_data.par_iter().for_each(|dd| {
            let dive = self.calc_dive_stats(&dd, &gas_mixes).unwrap();
            self.update_with_dive_data(dive);
        });
        Ok(())
    }

    fn from_dir(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let paths = Self::traverse_for_uddf(path)?;
        paths.par_iter().for_each(|path| {
            let UDDFData {
                dives_data,
                gas_mixes,
            } = self.extract_data_from_file(path.to_str().unwrap()).unwrap();
            dives_data.par_iter().for_each(|dd| {
                let dive = self.calc_dive_stats(&dd, &gas_mixes).unwrap();
                self.update_with_dive_data(dive);
            });
        });
        Ok(())
    }

    fn traverse_for_uddf(path: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut uddf_file_paths: Vec<PathBuf> = vec![];
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let mut traversal_res = Stats::traverse_for_uddf(path.to_str().unwrap())?;
                uddf_file_paths.append(&mut traversal_res);
            }
            let extension = path.extension().unwrap_or_default();
            if extension.to_ascii_lowercase() == "uddf" {
                uddf_file_paths.push(path);
            }
        }

        Ok(uddf_file_paths)
    }

    fn extract_data_from_file(&self, path: &str) -> Result<UDDFData, Box<dyn Error>> {
        // println!("Extracting dives from UDDF");
        let file = parser::parse_file(path)?;

        let gas_definitions = file.gas_definitions;
        let mut dives: Vec<DiveElem> = vec![];
        let repetition_groups = file.profile_data.repetition_group;
        for mut group in repetition_groups {
            dives.append(&mut group.dives);
        }

        Ok(UDDFData {
            gas_mixes: gas_definitions.gas_mixes,
            dives_data: dives,
        })
    }

    fn calc_dive_stats(
        &self,
        dive_data: &DiveElem,
        gas_mixes: &GasMixesData,
    ) -> Result<Dive, Box<dyn Error>> {
        // todo: set gradient factors from dive data with default fallback
        let tmp_init_gf = (30, 70);
        let tmp_treshold_depths: Vec<Depth> = vec![10., 20., 30., 40.];
        let mut dive = Dive::new(DiveConfig {
            gradient_factors: tmp_init_gf,
            treshold_depths: tmp_treshold_depths,
        });
        dive.calc_dive_stats(dive_data, gas_mixes);
        Ok(dive)
    }

    fn update_with_dive_data(&self, dive: Dive) {
        let stats_data_arc = Arc::clone(&self.stats_data);
        let mut stats_data = stats_data_arc.lock().unwrap();

        // dives no
        stats_data.dives_no += 1;
        // time
        stats_data.total_time += dive.total_time;
        // depth
        if dive.depth_max > stats_data.depth_max {
            stats_data.depth_max = dive.depth_max;
        }
        // time in deco
        if dive.time_in_deco > 0 {
            stats_data.time_in_deco += dive.time_in_deco;
            stats_data.deco_dives_no += 1;
        }
        // GFs
        if dive.gf_surf_max > stats_data.gf_surf_max {
            stats_data.gf_surf_max = dive.gf_surf_max;
        }
        if dive.gf_99_max > stats_data.gf_99_max {
            stats_data.gf_99_max = dive.gf_99_max;
        }
        if dive.gf_end > stats_data.gf_end_max {
            stats_data.gf_end_max = dive.gf_end;
        }
        // time below
        'outer: for dive_time_below in dive.time_below {
            let (dive_treshold_depth, dive_treshold_time) = dive_time_below;
            for global_time_below in &mut stats_data.time_below {
                let (global_treshold_depth, global_treshold_time) = global_time_below;
                if dive_treshold_depth == *global_treshold_depth {
                    global_time_below.1 += dive_treshold_time;
                    continue 'outer;
                }
            }
            stats_data
                .time_below
                .push((dive_treshold_depth, dive_treshold_time));
        }
    }

    pub fn print_to_console(&self) {
        let stats_data_arc = Arc::clone(&self.stats_data);
        let stats = stats_data_arc.lock().unwrap();

        println!("{}", "\n            STATS              ".underline());
        println!("Dives:              {}", Self::to_colored(stats.dives_no));
        println!(
            "Total time:         {}",
            Self::to_colored(Self::seconds_to_readable(stats.total_time))
        );
        println!(
            "Max depth:          {}{}",
            Self::to_colored(stats.depth_max),
            Self::to_colored("m")
        );
        println!(
            "Deco dives:         {}",
            Self::to_colored(stats.deco_dives_no)
        );
        println!(
            "Total time in deco: {}",
            Self::to_colored(Self::seconds_to_readable(stats.time_in_deco))
        );
        println!(
            "Max surface GF:     {}{}",
            Self::to_colored(stats.gf_surf_max.round()),
            Self::to_colored("%")
        );
        println!(
            "Max GF99:           {}{}",
            Self::to_colored(stats.gf_99_max.round()),
            Self::to_colored("%")
        );
        println!(
            "Max end GF:         {}{}",
            Self::to_colored(stats.gf_end_max.round()),
            Self::to_colored("%")
        );
        self.print_time_below(&stats.time_below);
    }

    fn to_colored<T: std::fmt::Display>(v: T) -> ColoredString {
        v.to_string().cyan().bold().dimmed()
    }

    fn print_time_below(&self, time_below: &TimeBelowDepthData) {
        println!("Time below:");
        for record in time_below.iter() {
            let (depth, time) = record;
            println!(
                "  - {}m:            {}",
                depth,
                Self::to_colored(Self::seconds_to_readable(*time))
            );
        }
    }

    pub fn seconds_to_readable(s: Seconds) -> String {
        let seconds = s % 60;
        let minutes = (s / 60) % 60;
        let hours = (s / 60) / 60;
        format!("{hours}h {minutes}m {seconds}s")
    }
}
