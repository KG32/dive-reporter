use dive_deco::{BuehlmannConfig, BuehlmannModel, DecoModel, Gas, Pressure, Supersaturation};

use crate::common::{GradientFactorsSetting, GF};
use crate::parser::WaypointElem;
use crate::stats::TimeBelowDepthData;
use crate::{common::{Depth, Seconds}, parser::DiveElem, stats::GasMixesData};

#[derive(Debug)]
pub struct DiveMeta {
    gradient_factors: GradientFactorsSetting,
    current_mix: Gas,
    last_depth: Depth,
}

#[derive(Debug)]
pub struct Dive {
    pub total_time: Seconds,
    pub depth_max: Depth,
    pub time_in_deco: Seconds,
    pub gf_surf_max: GF,
    pub gf_99_max: GF,
    pub gf_end: GF,
    pub time_below: TimeBelowDepthData,
    meta: DiveMeta,
}

pub struct DiveConfig {
    pub gradient_factors: GradientFactorsSetting,
    pub treshold_depths: Vec<Depth>,
}

impl Dive {
    pub fn new(config: DiveConfig) -> Dive {
        let init_gas = Gas::new(0.21, 0.);
        let dive_meta = DiveMeta {
            gradient_factors: config.gradient_factors,
            current_mix: init_gas,
            last_depth: 0.,
        };

        Dive {
            total_time: 0,
            depth_max: 0.0,
            time_in_deco: 0,
            gf_surf_max: 0.,
            gf_99_max: 0.,
            gf_end: 0.,
            time_below: Self::construct_treshold_depths(config.treshold_depths),
            meta: dive_meta,
        }
    }

    pub fn calc_dive_stats(&mut self, dive_data: &DiveElem, gas_mixes: &GasMixesData) {
        let (gf_lo, gf_hi) = self.meta.gradient_factors;
        let mut model = BuehlmannModel::new(BuehlmannConfig::new().gradient_factors(gf_lo, gf_hi));
        // calc by data point
        let mut last_waypoint_time: Seconds = 0;
        let dive_data_points = &dive_data.samples.waypoints;
        for data_point in dive_data_points {
            self.process_data_point(
                &mut model,
                &data_point,
                last_waypoint_time,
                &gas_mixes,
            );
            // update last waypoint time
            last_waypoint_time = data_point.dive_time;
        }
    }

    fn process_data_point(
        &mut self,
        model: &mut BuehlmannModel,
        data_point: &WaypointElem,
        last_waypoint_time: Seconds,
        gas_mixes: &GasMixesData,
    ) {
        // time
        let step_time: Seconds = data_point.dive_time - last_waypoint_time;
        self.total_time += step_time;

        // depth
        self.register_depth(&data_point.depth, &step_time);

        // check for gas switch
        let switchmix = &data_point.switchmix;
        match switchmix {
            Some(switchmix) => {
                let gas_ref = &switchmix.gas_ref;
                let gas = Self::gas_by_ref(gas_ref, gas_mixes);
                match gas {
                    Some(gas) => {
                        self.meta.current_mix = gas;
                    },
                    None => {
                        panic!("Gas not found");
                    }
                }
            },
            None => ()
        }

        // deco model step
        let gas = &self.meta.current_mix;
        model.step(data_point.depth, step_time, gas);

        // GFs
        let Supersaturation { gf_99, gf_surf } = model.supersaturation();
        self.register_gfs((gf_99, gf_surf), &step_time, &data_point.depth);

        // deco time
        if model.ceiling() > 0. {
            self.time_in_deco += step_time;
        }
    }

    fn register_depth(&mut self, depth: &Depth, step_time: &Seconds) {
        // max
        if depth > &self.depth_max {
            self.depth_max = *depth;
        }
        // treshold depths
        for time_below_item in &mut self.time_below {
            let (treshold_depth, mut current_time) = time_below_item;
            if depth >= treshold_depth {
                time_below_item.1 += step_time;
            }
        }
    }

    fn register_gfs(&mut self, gfs: (Pressure, Pressure), time: &Seconds, depth: &Depth) {
        let (gf_99, gf_surf) = gfs;
        // GF surf
        if gf_surf > self.gf_surf_max {
            self.gf_surf_max = gf_surf;
        }
        // GF99
        if gf_99 > self.gf_99_max {
            self.gf_99_max = gf_99;
        }
        // GF end
        if *depth == 0. && self.meta.last_depth != 0. {
            self.gf_end = gf_99;
        }
        // register last depth for end dive GF99 check
        self.meta.last_depth = *depth;
    }

    fn update_time_in_deco(&mut self, time: Seconds) {
        self.time_in_deco += time;
    }

    // @todo
    fn gas_by_ref(gas_ref: &str, gas_mixes: &GasMixesData) -> Option<Gas> {
        let mut gas: Option<Gas> = None;
        match gas_mixes {
            Some(gas_mixes) => {
                for mix_definition in gas_mixes {
                    if mix_definition.id == gas_ref {
                        gas = Some(Gas::new(mix_definition.o2, mix_definition.he.unwrap_or(0.)));
                    }
                }
            },
            None => {
                panic!("No gas mixes, can't process switch");
            }
        }
        gas
    }

    fn construct_treshold_depths(treshold_config: Vec<Depth>) -> TimeBelowDepthData {
        let mut time_below = vec![];
        for depth in treshold_config {
            time_below.push((depth, 0));
        }
        time_below
    }

}
