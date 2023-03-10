#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, RichText};
use rand::prelude::*;


#[derive(Default)]
pub struct MyApp {
    // display/input vars
    key: String,
    to_encrypt: String,
    encrypted_preview: String,
    encrypted_text: String,

    to_decrypt: String,
    decrypted_output: String,
    decryption_error: bool,
    // other
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
                let key = ui
                    .text_edit_singleline(&mut self.key)
                    .on_hover_text("This is your secret key.\nKeep it safe.")
                    .labelled_by(label.id);
                key_changed = key.changed();
            });
            ui.group(|ui| {
                ui.heading("Encrypt");
                let in_label = ui.label("Input text:");
                let input = ui
                    .text_edit_multiline(&mut self.to_encrypt)
                    .labelled_by(in_label.id);
                if input.changed() || key_changed {
                    self.encrypted_preview = {
                        let encrypted =
                            librcrypt::encrypt_raw(&self.key, 0, self.to_encrypt.as_bytes());
                        base64::encode(encrypted)
                    };
                    self.encrypted_text.clear();
                }

                let out_label = ui.label("Preview:");
                ui.label(&self.encrypted_preview)
                    .on_hover_text("Preview encrypted data (NOT THE REAL OUTPUT, and NOT SECURE!)")
                    .labelled_by(out_label.id);

                let out_label = ui.label("Encrypted Text:");
                ui.label(&self.encrypted_text).labelled_by(out_label.id);
                ui.horizontal(|ui| {
                    let enc_btn = ui.button("Encrypt");
                    if enc_btn.clicked() {
                        self.encrypted_text = librcrypt::encrypt_base64(
                            &self.key,
                            rand::rngs::OsRng.sample(rand::distributions::Uniform::new(0, 2 ^ 68)),
                            &self.to_encrypt,
                        );
                    }
                    let cpy_btn = ui.button("Copy encrypted");
                    if cpy_btn.clicked() {
                        ui.output().copied_text = self.encrypted_text.clone();
                    }
                    let clr_btn = ui.button("Clear");
                    if clr_btn.clicked() {
                        self.to_encrypt.clear();
                        self.encrypted_preview.clear();
                        self.encrypted_text.clear();
                    }
                });
            });
            ui.group(|ui| {
                ui.heading("Decrypt");
                let in_label = ui.label("Encrypted text:");
                let input = ui
                    .text_edit_multiline(&mut self.to_decrypt)
                    .labelled_by(in_label.id);
                if input.changed() || key_changed {
                    self.decryption_error = false;
                    self.decrypted_output =
                        match librcrypt::decrypt_base64(&self.key, &self.to_decrypt) {
                            Ok(text) => text,
                            Err(err) => {
                                use librcrypt::DecryptError::*;
                                self.decryption_error = true;
                                let mut f = format!("Error in decryption:\n{err:#?}");
                                match err {
                                    TooShort => f = "Too short to be decrypted".into(),
                                    InvalidUTF8(..) => f.push_str(
                                        "\nThis may be caused by a key mismatch, or mistyped input",
                                    ),
                                    LengthMismatch | NoMagic | Base64Decode(..) => {
                                        f.push_str("\nLikely a mistyped or invalid input")
                                    }
                                    Corrupt => f.push_str(
                                        "\nEither mistyped input or tampering was attempted!",
                                    ),
                                }
                                f
                            }
                        };
                }

                ui.label({
                    let r = RichText::new(&self.decrypted_output);
                    if self.decryption_error {
                        r.color(egui::Color32::RED)
                    } else {
                        r.color(egui::Color32::GREEN)
                    }
                });
            });
        });
    }
}
