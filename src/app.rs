use std::sync::{Mutex};
use eframe::{App, CreationContext, Frame};
use egui::Window;
use once_cell::sync::Lazy;
use crate::{get, init_static, render_pack_from_frame};
use crate::packages::time_package::TimePackage;
use crate::render_state::meh_renderer::MehRenderer;
use crate::render_state::structs::RenderPack;


// Globals
init_static!(TIME: TimePackage => {TimePackage::new()});


pub struct MehApp {
    meh_renderer: MehRenderer,
}
impl MehApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let renderer = &mut render_state.renderer.write();
        let device = &render_state.device;


        let meh_renderer = MehRenderer::new(device, renderer);


        Self {
            meh_renderer,
        }
    }

    fn update(&mut self, render_pack: &mut RenderPack<'_>) {
        get!(TIME).update();

        self.meh_renderer.update(render_pack)
    }

    fn render(&mut self, render_pack: &RenderPack<'_>) {
        self.meh_renderer.render_pass(render_pack)

    }

    fn ui(&mut self, ctx: &egui::Context) {

        Window::new("test")
            .resizable(true)
            .show(ctx, |ui| {
                self.meh_renderer.display(ui);
            });

        Window::new("time")
            .resizable(true)
            .show(ctx, |ui| {
                ui.label(format!("{}", get!(TIME).fps));
            });
    }
}

impl App for MehApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        render_pack_from_frame!(render_pack, frame);



        self.update(&mut render_pack);
        self.render(&render_pack);
        self.ui(ctx);

        ctx.request_repaint();
    }
}

