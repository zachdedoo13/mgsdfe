use eframe::egui::TextBuffer;

// basic
pub struct Oss {
   freq: f32,
   amp: f32,
   phase: f32,
}
enum FloatOrOss {
   Float(f32),
   Oss(Oss),
}
pub struct Float {
   val: FloatOrOss,
   id: u64,
}
pub struct Vec3 {
   x: Float,
   y: Float,
   z: Float,
}


// less basic
pub struct Transform {
   position: Vec3,
   rotation: Vec3,
   scale: Float,
}
pub struct Material {
   surface_color: Vec3,
}
pub struct Bounds {
   automatic: bool,
}


pub struct SDF {
   sdf_type: SdfType,
   settings: Vec3,
}
pub enum SdfType {
   Circle,
   Cube,

   Custom { data: String },
}


pub enum Layer {
   Shape {
      transform: Transform,
      material: Material,
      bounds: Bounds,
      sdf_type: SdfType,
   },
   Union {
      transform: Transform,
      bounds: Bounds,
      children: Vec<Layer>,
   },
   Mod,
}


pub enum PassType {
   BruteForce,
   AABB,
   BitwiseAABB,
   SmartAABB,
}
pub struct PassOptions {
   pass_type: PassType,
}

pub struct Passer {
   contents: Layer,
   pass_options: PassOptions,
}
impl Passer {
   pub fn new(contents: Layer, pass_options: PassOptions) -> Self {
      Self {
         contents,
         pass_options,
      }
   }

   pub fn pass(&mut self) -> String {
      match self.pass_options.pass_type {
         PassType::BruteForce => self.brute_force(),
         _ => todo!(),
      }
   }

   fn brute_force(&mut self) -> String {
      // init values
      let mut map = String::new();
      let mut cast = String::new();

      let mut depth = 0;
      let mut u_index = 0;

      // init code
      map.push_str({
         format!(r#"

      Hit map(vec3 p_in) {{
         // init
         Hit d0u0 = Hit(100000.0);
         vec3 d0t0 = p_in;

         // start

      "#).as_str()
      });

      cast.push_str({
         format!(r#"

      Hit cast(Ray ray) {{
         float t = 0.0;
         for (int i = 0; i < s.steps_per_ray; i++) {{
            vec3 p = ray.ro + ray.rd * t;
            Hit hit = map(p);
            t += hit.d;

            if (hit.d < MHD) break;
            if (t > FP) break;
         }}
         return Hit(t);
      }}

      "#).as_str()
      });


      // disclose functions
      fn disclose_layer(layer: &Layer, depth: i32, u_index: i32) -> String {
         let mut out = String::new();

         match layer {
            Layer::Shape {
               transform,
               material,
               bounds,
               sdf_type
            } => {

            }

            Layer::Union {
               transform,
               bounds,
               children
            } => {

            }

            Layer::Mod => todo!()
         }

         out
      }

      // disclose
      depth += 1;
      map.push_str(disclose_layer(&self.contents, depth, u_index).as_str());


      // cleanup

      map.push_str({
         format!(r#"

         return d0u0;
      }}

      "#).as_str()
      });


      return format!("{map}\n{cast}");
   }
}

// helper functions
fn disclose_transform(transform: Transform, depth: i32, u_index: i32) -> String {
   let mut out = String::new();


   out
}



#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_pass_brute_force() {
      let mut cid = 0;

      let mut get_cid = || {
         cid += 1;
         cid - 1
      };


      let data = Layer::Union {
         transform: Transform {
            position: Vec3 {
               x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
            },
            rotation: Vec3 {
               x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
            },
            scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
         },
         bounds: Bounds { automatic: false },
         children: vec![
            Layer::Shape {
               transform: Transform {
                  position: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
                  rotation: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
                  scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               },
               material: Material {
                  surface_color: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
               },
               bounds: Bounds { automatic: false },
               sdf_type: SdfType::Circle,
            }
         ],
      };

      let mut passer = Passer::new(data, PassOptions { pass_type: PassType::BruteForce });

      let out = passer.pass();
      println!("{out}");

      assert_eq!(1, 0);
   }
}

















