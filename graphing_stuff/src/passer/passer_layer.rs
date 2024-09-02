#![allow(dead_code)]
// basic
#[derive(Copy, Clone, Debug)]
pub struct Oss {
   freq: f32,
   amp: f32,
   phase: f32,
}
#[derive(Copy, Clone, Debug)]
enum FloatOrOss {
   Float(f32),
   Oss(Oss),
}
#[derive(Copy, Clone, Debug)]
pub struct Float {
   val: FloatOrOss,
   id: u64,
}
impl Float {
   fn is_zero(&self) -> bool {
      match self.val {
         FloatOrOss::Float(data) => !(data == 0.0),
         FloatOrOss::Oss(_) => false,
      }
   }

   fn comp(&self) -> String {
      match self.val {
         FloatOrOss::Float(data) => format!("({data})"),
         FloatOrOss::Oss(_) => todo!(),
      }
   }
}

#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
   x: Float,
   y: Float,
   z: Float,
}
impl Vec3 {
   fn is_zero(&self) -> bool {
      self.x.is_zero() | self.y.is_zero() | self.z.is_zero()
   }

   fn comp(&self) -> String {
      format!("vec3({}, {}, {})", self.x.comp(), self.y.comp(), self.z.comp())
   }
}


// less basic
pub struct Transform {
   position: Vec3,
   rotation: Vec3,
   scale: Float,
}
impl Transform {
   /// doesn't disclose scale
   fn comp_map<T: Into<String>>(&self, name: T, transform_reference: T) -> String {
      let name: String = name.into();
      let tfr: String = transform_reference.into();

      let pos = match self.position.is_zero() {
         true => format!("{name} = move({name}, {});", self.position.comp()),
         false => format!("//position zero"),
      };

      let rot = match self.rotation.is_zero() {
         true => format!("{name} *= rot3D({name}, {});", self.rotation.comp()),
         false => format!("//rotation zero"),
      };

      let scale = format!("{name} /= {};", self.scale.comp());

      format!(r#"
      vec3 {name} = {tfr};
      {scale}
      {pos}
      {rot}
      "#, )
   }

   fn scale_correction<T: Into<String>>(&self, apply_to: T) -> String {
      let sdf = apply_to.into();
      format!("{sdf}.d = scale_correction({sdf}.d, {});", self.scale.comp())
   }
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
impl SDF {
   fn comp_map<T: Into<String>>(&self, transform: T) -> String {
      let transform = transform.into();

      match self.sdf_type {
         SdfType::Sphere => format!("sdSphere({transform}, 1.0)"),

         SdfType::Cube => format!("sdCube({transform}, vec3(1.0, 1.0, 1.0)"),

         SdfType::Custom { .. } => todo!(),
      }
   }
}
pub enum SdfType {
   Sphere,
   Cube,

   Custom { data: String },
}


#[derive(Copy, Clone, Debug)]
pub struct Combination {
   comb: CombinationType,
   strength: Float,
}
impl Combination {
   fn comp_map<T: Into<String>>(&self, name: T, union_ref: T) -> String {
      let name = name.into();
      let union_ref = union_ref.into();
      match self.comb {
         CombinationType::Union => format!("{union_ref} = opUnion({name}, {union_ref});"),
         CombinationType::SmoothUnion => format!("{union_ref} = opSmoothUnion({name}, {union_ref}, {});", self.strength.comp()),
         CombinationType::Subtraction => todo!(),
         CombinationType::SmoothSubtraction => todo!(),
      }
   }
}
#[derive(Copy, Clone, Debug)]
pub enum CombinationType {
   Union,
   SmoothUnion,
   Subtraction,
   SmoothSubtraction,
}


pub enum Layer {
   Shape {
      transform: Transform,
      material: Material,
      bounds: Bounds,
      sdf: SDF,
   },
   Union {
      transform: Transform,
      bounds: Bounds,
      combination: Combination,
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

pub struct Passer<'a> {
   contents: &'a Layer,
   pass_options: PassOptions,
}
impl Passer<'_> {
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

      let upper_depth = 0;
      let upper_u_index = 0;

      // init code
      map.push_str({
         format!(r#"

      Hit map(vec3 p_in) {{
         // init
         Hit d0u0 = Hit(100000.0);
         vec3 t = p_in;

         // start

      "#).as_str()
      });

      cast.push_str({
         format!(r#"

      Hit cast_ray(Ray ray) {{
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
      #[derive(Debug)]
      struct Parcel {
         upper_union: String,
         upper_union_comb: Combination,
         upper_transform: String,
         depth: i32,
         u_index: i32,
      }

      fn disclose_shape(transform: &Transform, sdf: &SDF, parcel: &Parcel) -> String {
         let depth = parcel.depth;
         let u_index = parcel.u_index;
         let name = format!("d{depth}s{u_index}");

         let trans_name = format!("d{depth}u{}t", parcel.u_index);
         let trans = transform.comp_map(trans_name.clone(), parcel.upper_transform.clone());
         let trans = add_tabs_to_string(trans.as_str(), 3);
         let scale_cleanup = transform.scale_correction(name.clone());

         let sd = sdf.comp_map(trans_name);

         let close = parcel.upper_union_comb.comp_map(name.clone(), parcel.upper_union.to_string());

         let out = format!(r#"
         // shape
         {{
            {trans}

            Hit {name} = Hit({sd});

            // cleanup
            {scale_cleanup}
            {close}
         }}

         "#);

         add_tabs_to_string(out.as_str(), depth as usize)
      }

      fn disclose_union(transform: &Transform, combination: &Combination, children: &[Layer], parcel: &Parcel) -> String {
         let depth = parcel.depth;
         let u_index = parcel.u_index;
         let name = format!("d{depth}u{u_index}");

         let up = &parcel.upper_union;

         let trans_name = format!("d{depth}u{}t", parcel.u_index);
         let trans = transform.comp_map(trans_name.clone(), parcel.upper_transform.clone());
         let trans = add_tabs_to_string(trans.as_str(), 3);
         let scale_cleanup = transform.scale_correction(name.clone());

         let close = parcel.upper_union_comb.comp_map(name.clone(), parcel.upper_union.to_string());

         let mut childs = String::new();
         for (i, child) in children.iter().enumerate() {
            childs.push_str(disclose_layer(child, &Parcel {
               upper_union: name.clone(),
               upper_union_comb: combination.clone(),
               upper_transform: trans_name.clone(),
               depth: depth + 1,
               u_index: i as i32,
            }).as_str());
            childs.push('\n');
         };

         let out = format!(r#"
         // union
         {{
            // init and transform
            Hit d{depth}u{u_index} = {up};
            {trans}

            // children
            {{
               {childs}
            }}

            // cleanup
            {scale_cleanup}
            {close}
         }}
         "#);

         add_tabs_to_string(out.as_str(), depth as usize)
      }

      fn disclose_layer(layer: &Layer, parcel: &Parcel) -> String {
         match layer {
            Layer::Shape {
               transform,
               material: _,
               bounds: _,
               sdf,
            } => disclose_shape(transform, sdf, parcel),

            Layer::Union {
               transform,
               bounds: _,
               combination,
               children,
            } => disclose_union(transform, combination, children, parcel),

            Layer::Mod => todo!(),
         }
      }

      // disclose
      let upper = Parcel {
         upper_union: "d0u0".to_string(),
         upper_union_comb: Combination { comb: CombinationType::Union, strength: Float { val: FloatOrOss::Float(0.0), id: 0 } },
         upper_transform: "t".to_string(),
         depth: upper_depth + 1,
         u_index: upper_u_index,
      };

      map.push_str(disclose_layer(&self.contents, &upper).as_str());

      // cleanup
      map.push_str({
         format!(r#"

         return d0u0;
      }}

      "#).as_str()
      });

      format!("{map}\n{cast}")
   }
}




// helper functions
fn add_tabs_to_string(input: &str, tab_count: usize) -> String {
   let tabs = "\t".repeat(tab_count);
   input
       .lines()
       .map(|line| format!("{}{}", tabs, line))
       .collect::<Vec<String>>()
       .join("\n")
}
fn remove_tabs_from_string(input: &str, tab_count: usize) -> String {
   let tabs = "\t".repeat(tab_count);
   input
       .lines()
       .map(|line| {
          if line.starts_with(&tabs) {
             &line[tab_count..]
          } else {
             line
          }
       })
       .collect::<Vec<&str>>()
       .join("\n")
}
fn remove_spaces_from_string(input: &str, space_count: usize) -> String {
   let spaces = " ".repeat(space_count);
   input
       .lines()
       .map(|line| {
          if line.starts_with(&spaces) {
             &line[space_count..]
          } else {
             line
          }
       })
       .collect::<Vec<&str>>()
       .join("\n")
}

#[cfg(test)]
mod tests {
   use std::time::{Duration, Instant};

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
               y: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
               z: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
            },
            scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
         },
         bounds: Bounds { automatic: false },
         combination: Combination { comb: CombinationType::Union, strength: Float { val: FloatOrOss::Float(0.0), id: get_cid() } },
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
               sdf: SDF {
                  sdf_type: SdfType::Sphere,
                  settings: Vec3 {
                     x: Float { val: FloatOrOss::Float(5.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
               },
            },
            Layer::Union {
               transform: Transform {
                  position: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
                  rotation: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                  },
                  scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               },
               bounds: Bounds { automatic: false },
               combination: Combination { comb: CombinationType::Union, strength: Float { val: FloatOrOss::Float(1.0), id: get_cid() } },
               children: vec![],
            },
            Layer::Union {
               transform: Transform {
                  position: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
                  rotation: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                  },
                  scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               },
               bounds: Bounds { automatic: false },
               combination: Combination { comb: CombinationType::SmoothUnion, strength: Float { val: FloatOrOss::Float(2.8), id: get_cid() } },
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
                     sdf: SDF {
                        sdf_type: SdfType::Sphere,
                        settings: Vec3 {
                           x: Float { val: FloatOrOss::Float(5.0), id: get_cid() },
                           y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                           z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                        },
                     },
                  },
               ],
            },
         ],
      };

      let mut passer = Passer {
         contents: &data,
         pass_options: PassOptions { pass_type: PassType::BruteForce },
      };

      let out = passer.pass();
      println!("{out}");

      assert_eq!(1, 2);
   }

   #[test]
   fn test_speed() {
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
               y: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
               z: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
            },
            scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
         },
         bounds: Bounds { automatic: false },
         combination: Combination { comb: CombinationType::Union, strength: Float { val: FloatOrOss::Float(0.0), id: get_cid() } },
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
               sdf: SDF {
                  sdf_type: SdfType::Sphere,
                  settings: Vec3 {
                     x: Float { val: FloatOrOss::Float(5.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
               },
            },
            Layer::Union {
               transform: Transform {
                  position: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
                  rotation: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                  },
                  scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               },
               bounds: Bounds { automatic: false },
               combination: Combination { comb: CombinationType::Union, strength: Float { val: FloatOrOss::Float(1.0), id: get_cid() } },
               children: vec![],
            },
            Layer::Union {
               transform: Transform {
                  position: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                  },
                  rotation: Vec3 {
                     x: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                     y: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                     z: Float { val: FloatOrOss::Float(0.0), id: get_cid() },
                  },
                  scale: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
               },
               bounds: Bounds { automatic: false },
               combination: Combination { comb: CombinationType::SmoothUnion, strength: Float { val: FloatOrOss::Float(2.8), id: get_cid() } },
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
                     sdf: SDF {
                        sdf_type: SdfType::Sphere,
                        settings: Vec3 {
                           x: Float { val: FloatOrOss::Float(5.0), id: get_cid() },
                           y: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                           z: Float { val: FloatOrOss::Float(1.0), id: get_cid() },
                        },
                     },
                  },
               ],
            },
         ],
      };

      let mut comp_stop = vec![];

      for i in 0..10 {
         let mut tot = Duration::ZERO;

         let am = 100;
         for _ in 0..am {
            let st = Instant::now();
            let mut passer = Passer {
               contents: &data,
               pass_options: PassOptions { pass_type: PassType::BruteForce },
            };

            let out = passer.pass();
            comp_stop.push(out);

            tot += st.elapsed();
         }

         println!("AVE {i} -> {:?}", tot.div_f64(am as f64))
      }

      println!("total mem -> {}mb", comp_stop.iter().map(|e| string_memory_usage_mb(e)).sum::<f64>());

      assert_eq!(1, 1)
   }
}

fn string_memory_usage_mb(s: &String) -> f64 {
   const BYTES_PER_MB: f64 = 1_048_576.0; // 1024 * 1024 bytes in a megabyte
   const SSO_MAX_SIZE: usize = 23; // Typical max size for small string optimization

   let struct_size = std::mem::size_of::<String>();
   let content_size = if s.len() <= SSO_MAX_SIZE {
      0 // No heap allocation for small strings
   } else {
      s.capacity()
   };

   let total_bytes = struct_size + content_size;
   total_bytes as f64 / BYTES_PER_MB
}
















