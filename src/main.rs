// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui_wgpu::WgpuConfiguration;

#[cfg(not(target_arch = "wasm32"))]
use eframe::wgpu::PresentMode::Immediate;

#[cfg(not(target_arch = "wasm32"))]
use sdt_thing::app::MehApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        vsync: false,
        wgpu_options: WgpuConfiguration {
            present_mode: Immediate,
            power_preference: eframe::wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        },
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(MehApp::new(cc)))),
    )
}



// web
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::MehApp;


/// Your handle to the web app from JavaScript.
#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WebHandle {
    /// Installs a panic hook, then returns.
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Redirect [`log`] message to `console.log` and friends:
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Call this once from JavaScript to start your app.
    #[wasm_bindgen]
    pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
        self.runner
            .start(
                canvas_id,
                eframe::WebOptions {
                    wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                        power_preference: eframe::wgpu::PowerPreference::HighPerformance,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Box::new(|cc| Ok(Box::new(MehApp::new(cc))),)
            )
            .await
    }

    // The following are optional:

    /// Shut down eframe and clean up resources.
    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }

    /// The JavaScript can check whether or not your app has crashed:
    #[wasm_bindgen]
    pub fn has_panicked(&self) -> bool {
        self.runner.has_panicked()
    }

    #[wasm_bindgen]
    pub fn panic_message(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.message())
    }

    #[wasm_bindgen]
    pub fn panic_callstack(&self) -> Option<String> {
        self.runner.panic_summary().map(|s| s.callstack())
    }
}