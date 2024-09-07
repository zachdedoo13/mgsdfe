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

#[derive(Debug, Clone)]
pub struct Material {
   pub albedo: Vec3,
   pub emissive: Vec3,
   pub spec_chance: Float,
   pub spec_roughness: Float,
   pub index_of_reflection: Float,
   pub refraction_chance: Float,
   pub refraction_roughness: Float,
   pub refraction_color: Vec3,
}
impl Default for Material {
   fn default() -> Self {
      Self {
         albedo: Vec3::new_from_f32(1.0),
         emissive: Vec3::new_from_f32(0.0),
         spec_chance: Float::new(0.0),
         spec_roughness: Float::new(0.0),
         index_of_reflection: Float::new(1.0),
         refraction_chance: Float::new(0.0),
         refraction_roughness: Float::new(0.0),
         refraction_color: Vec3::new_from_f32(1.0),
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
