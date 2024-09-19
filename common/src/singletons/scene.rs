use bytemuck::{Pod, Zeroable};
use eframe::egui::{CollapsingHeader, DragValue, Ui};

/// used to hold all data for the node-graph and raymarching
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Scene {
   pub local_shapes: Vec<ShapeEntry>,
   pub active_cubemap: (),
   pub map_data: (),

   pub parthtrace_settings: ParthtracerSettings,
}
impl Default for Scene {
   fn default() -> Self {
      Self {
         local_shapes: vec![],
         active_cubemap: (),
         map_data: (),
         parthtrace_settings: ParthtracerSettings::default(),
      }
   }
}


///////////////////
// Shape storage //
///////////////////
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ShapeEntry {
   pub name: String,
   pub shader_code: String,
}
impl ShapeEntry {
   /// pre-made shapes, called inside a switch case, has the inputs (vec3 p) and (vec3 data)
   /// and (bool do_mat) and (Mat in_mat)
   pub fn hardcoded() -> Vec<ShapeEntry> {
      vec![
         // sphere
         ShapeEntry {
            name: "sphere".to_string(),
            shader_code: r#"
               return ShapeHit(length(p) - data.x, in_mat);
            "#.to_string(),
         },
      ]
   }
}


/////////////////////////
// Pathtracer settings //
/////////////////////////
#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ParthtracerSettings {
   pub time: f32,
   pub frame: i32,
   pub last_clear_frame: i32,

   pub samples_per_frame: i32,
   pub steps_per_ray: i32,
   pub bounces: i32,

   pub fov: f32,

   pub camera_pos: [f32; 3],
   pub camera_dir: [f32; 3],
}
impl Default for ParthtracerSettings {
   fn default() -> Self {
      Self {
         time: 0.0,
         frame: 0,
         last_clear_frame: 0,
         samples_per_frame: 0,
         steps_per_ray: 80,
         bounces: 8,
         fov: 1.0,
         camera_pos: [0.0, 0.0, 0.0],
         camera_dir: [0.0, 0.0, 0.0],
      }
   }
}
impl ParthtracerSettings {
   pub fn ui(&mut self, ui: &mut Ui) {
      ui.group(|ui| {
         CollapsingHeader::new("Variables").show(ui, |ui| {
                ui.label(format!("Time -> {}", self.time));
                ui.label(format!("Frame -> {}", self.frame));
                ui.label(format!("Last clear frame -> {}", self.last_clear_frame));
             });

         ui.group(|ui| {
            ui.label("Render");
            ui.horizontal(|ui| {
               ui.add(DragValue::new(&mut self.samples_per_frame).range(1..=16).speed(0.01).prefix("Samples: "));
               ui.add(DragValue::new(&mut self.steps_per_ray).range(1..=320).speed(0.1).prefix("Steps: "));
               ui.add(DragValue::new(&mut self.bounces).range(0..=32).speed(0.1).prefix("Bounces: "));
            });
            ui.horizontal(|ui| {
               ui.add(DragValue::new(&mut self.fov).range(0.0..=10.0).speed(0.001).prefix("FOV"));
            })
         });

         ui.group(|ui| {
            ui.label("Camera position");
            ui.horizontal(|ui| {
               ui.add(DragValue::new(&mut self.camera_pos[0]).range(-100.0..=100.0).speed(0.01).prefix("X: "));
               ui.add(DragValue::new(&mut self.camera_pos[1]).range(-100.0..=100.0).speed(0.01).prefix("Y: "));
               ui.add(DragValue::new(&mut self.camera_pos[2]).range(-100.0..=100.0).speed(0.01).prefix("Z: "));
            });
         });

      });
   }
}