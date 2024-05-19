#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, InnerResponse, Ui};
use std::future::Future;
use std::error::Error;
use rfd::FileDialog;
use crate::{dive, stats::{Stats, StatsOutput}};

pub struct App {
    title: String,
    stats: Stats,
    stats_output: StatsOutput,
    config: AppConfig,
    state: AppState,
}

struct AppState {
    error: Option<AppError>
}

struct AppError {
    text: String,
}

struct AppConfig {
    path: Option<String>,
    gradient_factors: (u8, u8),
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: "Dive reporter".to_owned(),
            stats: Stats::new(),
            stats_output: vec![],
            config: AppConfig {
                path: None,
                gradient_factors: (30, 70)
            },
            state: AppState {
                error: None,
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.title);
            ui.separator();

            // config
            self.render_pair(ui, "Path:", &self.config.path.clone().unwrap_or("-".to_string()));
            ui.separator();

            // open file btn
            if ui.button("📂 Open UDDF file").clicked() {
                let file = FileDialog::new()
                    .set_directory("/")
                    .pick_file();

                if let Some(file_path) = file {
                    // todo err handling
                    let selected_path = file_path.to_str().unwrap();
                    self.update_path(selected_path.to_string());
                    let stats_res = Stats::new().from_path(file_path.to_str().unwrap());
                    match stats_res {
                        Ok(stats) => { self.stats = stats },
                        Err(err) => { self.handle_stats_error(ui, err) }
                    }
                }
            }
            ui.separator();

            // stats container
            let stats = &self.stats;
            if stats.dives_no > 0 {
                let depth_max = stats.depth_max.to_string();
                let gf_surf_max = stats.gf_surf_max.round().to_string();
                let gf_99_max = stats.gf_99_max.round().to_string();
                let gf_end_max = stats.gf_end_max.round().to_string();

                ui.vertical(|ui| {
                    self.render_pair(ui, "Dives:", &stats.dives_no.to_string());
                    self.render_pair(ui, "Total time:", &Stats::seconds_to_readable(stats.total_time));
                    self.render_pair(ui, "Max depth", &format!("{depth_max}m"));
                    self.render_pair(ui, "Deco dives:", &stats.deco_dives_no.to_string());
                    self.render_pair(ui, "Total time in deco:", &Stats::seconds_to_readable(stats.time_in_deco));
                    self.render_pair(ui, "Max surface GF:", &format!("{gf_surf_max}%"));
                    self.render_pair(ui, "Max GF99:", &format!("{gf_99_max}%"));
                    self.render_pair(ui, "Max end GF:", &format!("{gf_end_max}%"));
                    self.render_pair(ui, "Time below:", "");
                    for record in stats.time_below.iter() {
                        let (depth, time) = record;
                        ui.indent("", |ui| {
                            self.render_pair(ui, &format!("-{depth}:"), &Stats::seconds_to_readable(*time));
                        });
                    }
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

    fn handle_stats_error(&self, ui: &mut Ui, err: Box<dyn Error>) {
        println!("Stats error!");
        println!("{}", err);
    }

    fn update_path(&mut self, new_path: String) {
        self.config.path = Some(new_path);
    }

    pub fn render_pair(&self, ui: &mut Ui, v1: &str, v2: &str) -> InnerResponse<()> {
        ui.horizontal(|ui| {
            ui.label(v1);
            ui.label(v2);
        })
    }
}
