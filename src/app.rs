#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, InnerResponse, Ui};
use std::future::Future;
use rfd::FileDialog;
use crate::{dive, stats::{Stats, StatsOutput}};

pub struct App {
    title: String,
    stats: Stats,
    stats_output: StatsOutput,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "Dive reporter".to_owned(),
            stats: Stats::new(),
            stats_output: vec![],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.title);
            // a simple button opening the dialog
            if ui.button("📂 Open UDDF file").clicked() {
                let file = FileDialog::new()
                .set_directory("/")
                .pick_file();

                if let Some(file_path) = file {
                    // todo err handling
                    self.stats = Stats::new().from_path(file_path.to_str().unwrap()).unwrap();
                }
            }

            let stats = &self.stats;
            if stats.dives_no > 0 {
                ui.vertical(|ui| {
                    self.render_pair(ui, "Dives:", &stats.dives_no.to_string());
                    self.render_pair(ui, "Total time:", &Stats::seconds_to_readable(stats.total_time));
                    self.render_pair(ui, "Max depth", &stats.depth_max.to_string());
                    self.render_pair(ui, "Deco dives:", &stats.deco_dives_no.to_string());
                    self.render_pair(ui, "Total time in deco:", &&Stats::seconds_to_readable(stats.time_in_deco));
                    self.render_pair(ui, "Max surface GF:", &stats.gf_surf_max.round().to_string());
                    self.render_pair(ui, "Max GF99:", &stats.gf_99_max.round().to_string());
                    self.render_pair(ui, "Max end GF:", &stats.gf_end_max.round().to_string());
                });
            }
        });
    }
}

impl App {
    pub fn init(&self) -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 600.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Dive reporter",
            options,
            Box::new(|_cc| {
                Box::<App>::default()
            }),
        )
    }

    pub fn render_pair(&self, ui: &mut Ui, v1: &str, v2: &str) -> InnerResponse<()> {
        ui.horizontal(|ui| {
            ui.label(v1);
            ui.label(v2);
        })
    }
}
