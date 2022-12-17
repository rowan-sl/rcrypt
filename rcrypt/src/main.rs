#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "RCrypt v4.2.0",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    )
}

#[derive(Default)]
struct MyApp {
    key: String,
    to_encrypt: String,
    encrypted: String,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // the logical ordering of this (might be) fragile at best
            // but it appears like egui runs the closures passed to ui.group()
            // in order, so it works for now.
            let mut key_changed = false;
            ui.group(|ui| {
                ui.heading("Encryption Settings");
                let label = ui.label("Secret Key");
                let key = ui.text_edit_singleline(&mut self.key)
                    .on_hover_text("This is your secret key.\nKeep it safe.")
                    .labelled_by(label.id);
                key_changed = key.changed();
            });
            ui.group(|ui| {
                ui.heading("Encrypt");
                let in_label = ui.label("Input text:");
                let input = ui.text_edit_multiline(&mut self.to_encrypt)
                    .labelled_by(in_label.id);
                if input.changed() || key_changed {
                    // self.encrypted = librcrypt::encode_base64(&self.key, &self.to_encrypt)
                }

                let out_label = ui.label("Preview encryption:");
                ui.label(&self.encrypted)
                    .labelled_by(out_label.id);
            })
        });
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.heading("My egui Application");
        //     ui.horizontal(|ui| {
        //         let name_label = ui.label("Your name: ");
        //         ui.text_edit_singleline(&mut self.name)
        //             .labelled_by(name_label.id);
        //     });
        //     ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
        //     if ui.button("Click each year").clicked() {
        //         self.age += 1;
        //     }
        //     ui.label(format!("Hello '{}', age {}", self.name, self.age));
        // });
    }
}
