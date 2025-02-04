// Disable Windows console on release builds
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() {
    // prepare() checks if it's a single instance and tries to send the args otherwise.
    // It should always be the first line in your main function (with the exception of loggers or similar)
    cargo_deep_link::prepare("de.fabianlars.deep-link-test");

    env_logger::init();

    cargo_deep_link::listen(|str| {
        println!("{:?}", str);
    })
    .unwrap();

    eframe::run_native(
        "egui App",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::<App>::default())),
    )
    .unwrap();
}

struct App {
    args: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            args: std::env::args().collect::<Vec<_>>().join(" "),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui Application");

            ui.label(self.args.clone());

            if ui.add(egui::Button::new("Register")).clicked() {
                // If you need macOS support this must be called in .setup() !
                // Otherwise this could be called right after prepare() but then you don't have access to tauri APIs
                cargo_deep_link::register(
                    "test-scheme",
                    |request| {
                       println!("{:?}", &request);
                    },
                )
                .unwrap(/* If listening to the scheme is optional for your app, you don't want to unwrap here. */);
            }

            if ui.add(egui::Button::new("Unregister")).clicked() {
                cargo_deep_link::unregister("test-scheme").unwrap();
            }
        });
    }
}
