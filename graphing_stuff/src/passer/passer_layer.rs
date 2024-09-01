use eframe::egui::TextBuffer;

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
   fn comp<T: Into<String>>(&self, name: T, transform_reference: T) -> String {
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

      format!(r#"
      vec3 {name} = {tfr};
      {pos}
      {rot}
      "#, )
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
pub enum SdfType {
   Circle,
   Cube,

   Custom { data: String },
}


#[derive(Copy, Clone, Debug)]
pub struct Combination {
   comb: CombinationType,
   strength: Float,
}
impl Combination {
   fn comp<T: Into<String>>(&self, name: T, union_ref: T) -> String {
      let name = name.into();
      let union_ref = union_ref.into();
      match self.comb {
         CombinationType::Union => format!("{union_ref} = opUnion({name}, {union_ref});"),
         CombinationType::SmoothUnion => todo!(),
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
      sdf_type: SdfType,
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

      let mut upper_depth = 0;
      let mut upper_u_index = 0;

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
      #[derive(Debug)]
      struct Parcel {
         upper_union: String,
         upper_union_comb: Combination,

         upper_transform: String,

         depth: i32,
         u_index: i32,
      }

      fn disclose_layer(layer: &Layer, parcel: &Parcel) -> String {
         let out: String = match layer {
            Layer::Shape {
               transform,
               material,
               bounds,
               sdf_type
            } => {
               format!("{parcel:?}")
            }

            Layer::Union {
               transform,
               bounds: _bounds,
               combination,
               children
            } => {
               let depth = parcel.depth;
               let u_index = parcel.u_index;
               let name = format!("d{depth}u{u_index}");

               let up = &parcel.upper_union;

               let trans_name = format!("u{}t", parcel.u_index);
               let trans = transform.comp(trans_name.clone(), parcel.upper_transform.clone());
               let trans = add_tabs_to_string(trans.as_str(), 3);

               let close = combination.comp(name.clone(), format!("{}", parcel.upper_union));

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

               format!(r#"

               {{
                  // init and transform
                  Hit d{depth}u{u_index} = {up};
                  {trans}


                  // children
                  {{
                     {childs}
                  }}

                  // cleanup
                  {close}
               }}

               "#, )
            }

            Layer::Mod => todo!()
         };


         out
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


      return format!("{map}\n{cast}");
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

















