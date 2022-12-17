#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;
use eframe::egui;
use rand::prelude::*;
use std::io::Read;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, clap::Parser)]
pub struct Args {
    /// simple stdin -> librcrypt -> stdout encryption and decryption.
    #[command(subcommand)]
    cmd: Option<Cmd>,
    /// Key to use with non-GUI operations
    #[clap(long, short)]
    key: Option<String>,
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, clap::Subcommand)]
pub enum Cmd {
    Encrypt,
    Decrypt,
}
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let args = Args::parse();
    if args.cmd.is_some() {
        assert!(
            args.key.is_some(),
            "CLI operation used, but no key provided!"
        );
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).unwrap();
        match args.cmd.unwrap() {
            Cmd::Encrypt => {
                print!(
                    "{}",
                    librcrypt::encrypt_base64(
                        &args.key.unwrap(),
                        rand::rngs::OsRng.sample(rand::distributions::Uniform::new(0, 2 ^ 68)),
                        &buf
                    )
                )
            }
            Cmd::Decrypt => match librcrypt::decrypt_base64(&args.key.unwrap(), &buf) {
                Ok(txt) => print!("{txt}"),
                Err(err) => {
                    use librcrypt::DecryptError::*;
                    let mut f = format!("Error in decryption:\n{err:#?}");
                    match err {
                        TooShort => f = "Too short to be decrypted".into(),
                        InvalidUTF8(..) => {
                            f.push_str("\nThis may be caused by a key mismatch, or mistyped input")
                        }
                        LengthMismatch | NoMagic | Base64Decode(..) => {
                            f.push_str("\nLikely a mistyped or invalid input")
                        }
                        Corrupt => {
                            f.push_str("\nEither mistyped input or tampering was attempted!")
                        }
                    }
                    eprintln!("{f}")
                }
            },
        }
    } else {
        // Log to stdout (if you run with `RUST_LOG=debug`).
        tracing_subscriber::fmt::init();

        let options = eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(320.0, 240.0)),
            ..Default::default()
        };
        eframe::run_native(
            "RCrypt v4.2.0",
            options,
            Box::new(|_cc| Box::new(rcrypt::MyApp::default())),
        )
    }
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|_cc| Box::new(rcrypt::MyApp::default())),
        )
        .await
        .expect("failed to start eframe");
    });
}
