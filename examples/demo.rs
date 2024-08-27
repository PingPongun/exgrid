// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use exgrid::ExGrid;
use exgrid::GridMode;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([600.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "ExGrid example",
        options,
        Box::new(|_| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Default)]
struct MyApp {}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ExGrid::new("some_unique_id")
                    .mode(GridMode::CompactWidth)
                    // .mode(GridMode::Traditional)
                    .show(ui, |ui| {
                        ui.start_collapsing();
                        ui.extext("Data");
                        // ui.extext("Data");
                        ui.end_row();
                        ui.extext("First row, first column");
                        ui.extext("First row, second column");
                        ui.end_row();

                        ui.extext("Second row, first column");
                        ui.extext("Second row, second column");
                        // ui.end_row_weak();
                        ui.extext("Second row, third column");
                        ui.end_row();

                        ui.horizontal(|ui| {
                            ui.label("Same");
                            ui.label("cell");
                        });
                        ui.extext("Third row, second column");
                        ui.end_row();
                        ui.collapsing_rows(|ui| {
                            ui.extext("collapsing header");
                            ui.extext("collapsing header-col2");
                            ui.extext("collapsing header-col3")
                        })
                        .body(|ui| {
                            ui.extext("collapsing row 1-col 1");
                            ui.extext("collapsing row 1-col2");
                            ui.extext("collapsing row 1-col3");
                            ui.end_row();
                            ui.extext("collapsing row 2-col 1");
                            ui.extext("collapsing row 2-col2");
                            ui.extext("collapsing row 2-col3");
                            ui.end_row();
                            ui.start_collapsing();
                            ui.extext("double nested header");
                            ui.extext("double nested header-col2");
                            ui.end_row();

                            ui.extext("double nested row 1-col 1");
                            ui.extext("double nested row 1-col2");
                            ui.extext("double nested row 1-col3");
                            ui.end_row();
                            ui.extext("double nested row 2-col 1");
                            ui.extext("double nested row 2-col2");
                            ui.extext("double nested row 2-col3");
                            ui.stop_collapsing();
                            ui.extext("collapsing row 3-col 1");
                            ui.extext("collapsing row 3-col2");
                            ui.extext("collapsing row 3-col3")
                        });
                        ui.stop_collapsing();
                    });
            });
        });
    }
}
