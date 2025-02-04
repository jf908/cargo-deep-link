// Disable Windows console on release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{
    Arc, Mutex,
};

use eframe::egui::{self, ViewportCommand};

fn main() {
    // prepare() checks if it's a single instance and tries to send the args otherwise.
    // It should always be the first line in your main function (with the exception of loggers or similar)
    cargo_deep_link::prepare("de.fabianlars.deep-link-test");

    env_logger::init();

    eframe::run_native(
        "egui App",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            let frame = cc.egui_ctx.clone();
            let last_call = Arc::new(Mutex::new(None));

            {
                let last_call = last_call.clone();

                cargo_deep_link::listen(move |str| {
                    last_call.lock().unwrap().replace(str);
                    frame.send_viewport_cmd(ViewportCommand::Focus);
                })
                .unwrap();
            }

            Ok(Box::new(App {
                last_call,
                ..Default::default()
            }))
        }),
    )
    .unwrap();
}

struct App {
    args: String,
    last_call: Arc<Mutex<Option<String>>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            args: std::env::args().collect::<Vec<_>>().join(" "),
            last_call: Arc::new(Mutex::new(None)),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui Application");

            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Register")).clicked() {
                    // If you need macOS support this must be called in .setup() !
                    // Otherwise this could be called right after prepare() but then you don't have access to tauri APIs
                    cargo_deep_link::register("test-scheme").unwrap(/* If listening to the scheme is optional for your app, you don't want to unwrap here. */);
                }

                if ui.add(egui::Button::new("Unregister")).clicked() {
                    cargo_deep_link::unregister("test-scheme").unwrap();
                }
            });

            ui.separator();
            
            ui.label("Args:");

            ui.text_edit_singleline(&mut self.args.clone());
            
            ui.label("Last link:");
            ui.text_edit_singleline(&mut self.last_call.lock().unwrap().as_ref().cloned().unwrap_or_default());
        });
    }
}
