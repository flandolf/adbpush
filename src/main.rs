#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, IconData};
use image::GenericImageView;
use std::process::Command;
mod icon;
fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 800.0])
            .with_drag_and_drop(true)
            .with_icon(load_icon()),

        ..Default::default()
    };
    eframe::run_native(
        "ADB Push",
        options,
        Box::new(|_cc| Ok(Box::<AdbPush>::default())),
    )
}
fn load_icon() -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::load_from_memory(icon::ICON).unwrap();
        let (width, height) = image.dimensions();
        let rgba = image.into_rgba8();
        (rgba, width, height)
    };

    IconData {
        rgba: icon_rgba.to_vec(),
        width: icon_width,
        height: icon_height,
    }
}
#[derive(Default)]
struct AdbPush {
    dropped_files: Vec<egui::DroppedFile>,
    target_path: String,
    device: String,
    init_device: bool,
    output: Vec<String>,
}

impl eframe::App for AdbPush {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.init_device {
            self.device = refresh_device();
            self.init_device = true;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ADB Push - File Transfer Tool");
            ui.label("Larger Files may cause app to freeze temporarily.");

            // Device status section
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Device Connected:");
                    ui.strong(&self.device);
                    if ui.button("Refresh").clicked() {
                        self.device = refresh_device();
                    }
                });
            });

            ui.separator();

            // Dropped files section
            egui::CollapsingHeader::new("Dropped Files")
                .default_open(true)
                .show(ui, |ui| {
                    if !self.dropped_files.is_empty() {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for file in &self.dropped_files {
                                    if let Some(path) = &file.path {
                                        ui.label(format!("{}", path.display()));
                                    }
                                }
                            });
                        if ui.button("Clear files").clicked() {
                            self.dropped_files.clear();
                        }
                    } else {
                        ui.label("No files dropped yet.");
                    }
                });

            ui.separator();

            // Target path and send controls
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("/storage/emulated/0/");
                    ui.text_edit_singleline(&mut self.target_path)
                        .on_hover_text("Enter target path");
                    if ui.button("Send").clicked() {
                        if self.dropped_files.is_empty() {
                            self.output.push("No files to send.".to_string());
                        } else if self.device == "No valid device." {
                            self.output.push("No valid device connected.".to_string());
                        } else {
                            self.send_files();
                        }
                    }
                });
            });

            ui.separator();

            // Output logs
            egui::CollapsingHeader::new("Output Logs")
                .default_open(true)
                .show(ui, |ui| {
                    if !self.output.is_empty() {
                        egui::ScrollArea::vertical()
                            .max_height(200.0)
                            .show(ui, |ui| {
                                for line in &self.output {
                                    ui.monospace(line);
                                }
                            });
                        if ui.button("Clear Output").clicked() {
                            self.output.clear();
                        }
                    } else {
                        ui.label("No logs yet.");
                    }
                });

            // Handle file drops
            ctx.input(|i| {
                if !i.raw.dropped_files.is_empty() {
                    for file in &i.raw.dropped_files {
                        if let Some(path) = &file.path {
                            if path.is_dir() {
                                self.output
                                    .push(format!("{} is a directory", path.display()));
                            } else {
                                self.dropped_files.push(file.clone());
                            }
                        }
                    }
                }
            });
        });
    }
}

impl AdbPush {
    fn send_files(&mut self) {
        for file in &self.dropped_files {
            if let Some(file_path) = &file.path {
                let target = format!("/storage/emulated/0/{}", self.target_path);

                let output = Command::new("adb")
                    .arg("push")
                    .arg(file_path)
                    .arg(&target)
                    .output();

                match output {
                    Ok(output) => {
                        let result = format!(
                            "Sent {:?} to {:?}: {}",
                            file_path,
                            target,
                            String::from_utf8_lossy(&output.stdout)
                        );
                        self.output.push(result);
                    }
                    Err(err) => {
                        self.output
                            .push(format!("Failed to send {:?}: {}", file_path, err));
                    }
                }
            } else {
                self.output.push("File path not found.".to_string());
            }
        }

        self.dropped_files.clear(); // Clear files after sending
    }
}
fn refresh_device() -> String {
    let output = Command::new("adb")
        .arg("devices")
        .output()
        .expect("Failed to execute command");
    let output = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<&str> = output.lines().collect();

    if devices.len() > 1 {
        return devices[1]
            .split_whitespace()
            .next()
            .unwrap_or("No valid device")
            .to_string();
    } else {
        return "No devices found".to_string();
    }
}
