use buehlmann_deco::{DecoModel, Gas, Step, Pressure};

use crate::common::{GradientFactorsSetting, GF};
use crate::parser::WaypointElem;
use crate::{common::{Depth, Seconds}, parser::DiveElem, stats::GasMixesData};

#[derive(Debug)]
pub struct DiveMeta {
    gradient_factors: GradientFactorsSetting,
    current_mix: Gas,
}

#[derive(Debug)]
pub struct Dive {
    pub total_time: Seconds,
    pub depth_max: Depth,
    pub time_in_deco: Seconds,
    pub gf_surf_max: GF,
    pub gf_end: GF,
    meta: DiveMeta,
}

impl Dive {
    pub fn new() -> Dive {
        let init_gas = Gas::new(0.21, 0.);
        let init_gradient_factors: GradientFactorsSetting = (100., 100.);
        let dive_meta = DiveMeta {
            gradient_factors: init_gradient_factors,
            current_mix: init_gas,
        };

        Dive {
            total_time: 0,
            depth_max: 0.0,
            time_in_deco: 0,
            gf_surf_max: 0.,
            gf_end: 0.,
            meta: dive_meta,
        }
    }

    pub fn calc_dive_stats(&mut self, dive_data: &DiveElem, gas_mixes: &GasMixesData) {
        let mut model = DecoModel::new();
        // calc by data point
        let mut last_waypoint_time: usize = 0;
        let dive_data_points = &dive_data.samples.waypoints;
        for data_point in dive_data_points {
            self.process_data_point(
                &mut model,
                &data_point,
                &last_waypoint_time,
                &gas_mixes,
            );
            // update last waypoint time
            last_waypoint_time = data_point.dive_time;
        }

        // end dive GF
        let (gf_end, ..) = model.gfs_current();
        self.gf_end = gf_end;
    }

    fn process_data_point(
        &mut self,
        model: &mut DecoModel,
        data_point: &WaypointElem,
        last_waypoint_time: &usize,
        gas_mixes: &GasMixesData,
    ) {
        // depth
        self.register_depth(&data_point.depth);

        // time
        let step_time = data_point.dive_time - last_waypoint_time;
        self.total_time += step_time;

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
        model.step(&data_point.depth, &step_time, gas);

        // GFs
        let gfs = model.gfs_current();
        self.register_gfs(gfs, &step_time);

    }

    fn register_depth(&mut self, depth: &Depth) {
        // max
        if depth > &self.depth_max {
            self.depth_max = *depth;
        }
    }

    fn register_gfs(&mut self, gfs: (Pressure, Pressure), time: &Seconds) {
        let (gf_now, gf_surf) = gfs;
        self.gf_end = gf_now;
        // GF surf
        if gf_surf > self.gf_surf_max {
            self.gf_surf_max = gf_surf;
        }
        // deco
        let (.., gf_high) = self.meta.gradient_factors;
        if gf_surf > gf_high {
            self.time_in_deco += time;
        }
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
}
