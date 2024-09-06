// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release


use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui_wgpu::WgpuConfiguration;
// web
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::{Adapter, DeviceDescriptor, Features};
#[cfg(not(target_arch = "wasm32"))]
use wgpu::PresentMode::Immediate;

#[cfg(not(target_arch = "wasm32"))]
use app::app::MehApp;

#[cfg(target_arch = "wasm32")]
use crate::MehApp;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    println!("remember to hide console in releases");


    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let device_descriptor_fn: Arc<dyn Fn(&Adapter) -> DeviceDescriptor<'static>> = Arc::new(|_adapter: &Adapter| {
        DeviceDescriptor {
            label: Some("wgpu native device desc"),
            required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            required_limits: Default::default(),
        }
    });

    let native_options = eframe::NativeOptions {
        vsync: false,
        wgpu_options: WgpuConfiguration {
            present_mode: Immediate,
            power_preference: wgpu::PowerPreference::HighPerformance,
            device_descriptor: device_descriptor_fn,
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
        let device_descriptor_fn: Arc<dyn Fn(&Adapter) -> DeviceDescriptor<'static>> = Arc::new(|_adapter: &Adapter| {
            DeviceDescriptor {
                label: Some("wgpu native device desc"),
                required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                required_limits: Default::default(),
            }
        });


        self.runner
            .start(
                canvas_id,
                eframe::WebOptions {
                    wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
                        power_preference: eframe::wgpu::PowerPreference::HighPerformance,
                        device_descriptor: device_descriptor_fn,
                        supported_backends: wgpu::Backends::BROWSER_WEBGPU,
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