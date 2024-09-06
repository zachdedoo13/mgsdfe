use crate::datastructures::{Float, Vec3};

#[derive(Debug)]
pub struct Transform {
   pub position: Vec3,
   pub rotation: Vec3,
   pub scale: Float,
}
impl Transform {
   /// doesn't disclose scale
   pub fn comp_map<T: Into<String>>(&self, name: T, transform_reference: T) -> String {
      let name: String = name.into();
      let tfr: String = transform_reference.into();

      let pos = match self.position.is_zero() {
         true => format!("{name} = move({name}, {} * (1.0 / {}));", self.position.comp(), self.scale.comp()),
         false => format!("//position zero"),
      };

      let rot = match self.rotation.is_zero() {
         true => format!("{name} = rot3D({name}, {});", self.rotation.comp()),
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

   pub fn scale_correction<T: Into<String>>(&self, apply_to: T) -> String {
      let sdf = apply_to.into();
      format!("{sdf}.d = scale_correction({sdf}.d, {});", self.scale.comp())
   }
}
impl Default for Transform {
   fn default() -> Self {
      Self {
         position: Default::default(),
         rotation: Default::default(),
         scale: Default::default(),
      }
   }
}

#[derive(Debug)]
pub struct Material {
   pub surface_color: Vec3,
}
impl Default for Material {
   fn default() -> Self {
      Self {
         surface_color: Default::default(),
      }
   }
}

#[derive(Debug)]
pub struct Bounds {
   pub automatic: bool,
}
impl Default for Bounds {
   fn default() -> Self {
      Self {
         automatic: true,
      }
   }
}
