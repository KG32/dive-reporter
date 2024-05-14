#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
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
                    self.stats = Stats::new().from_path(file_path.to_str().unwrap()).unwrap();
                    self.stats_output = vec![];
                    self.stats_output.push(("Total dives".to_string(), self.stats.dives_no.to_string()));
                    self.stats_output.push(("Total time".to_string(), self.stats.total_time.to_string()));
                }
            }

            ui.vertical(|ui| {
                for stat in &self.stats_output {
                    ui.horizontal(|ui| {
                        ui.label(stat.0.to_owned());
                        ui.label(stat.1.to_owned());
                    });
                }
            });
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
}
