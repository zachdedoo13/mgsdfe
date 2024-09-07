#[derive(Copy, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Oss {
   pub freq: f32,
   pub amp: f32,
   pub phase: f32,
}

#[derive(Copy, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum FloatOrOss {
   Float(f32),
   Oss(Oss),
}

#[derive(Copy, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Float {
   pub val: FloatOrOss,
   pub id: u64,
}
impl Float {
   pub fn is_zero(&self) -> bool {
      match self.val {
         FloatOrOss::Float(data) => !(data == 0.0),
         FloatOrOss::Oss(_) => false,
      }
   }

   pub fn comp(&self) -> String {
      match self.val {
         FloatOrOss::Float(data) => format!("({data})"),
         FloatOrOss::Oss(_) => todo!(),
      }
   }

   pub fn new(val: f32) -> Self {
      Self {
         val: FloatOrOss::Float(val),
         id: 0,
      }
   }
}
impl Default for Float {
   fn default() -> Self {
      Self {
         val: FloatOrOss::Float(1.0),
         id: 0,
      }
   }
}


#[derive(Copy, Clone, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Vec3 {
   pub x: Float,
   pub y: Float,
   pub z: Float,
}
impl Vec3 {
   pub fn is_zero(&self) -> bool {
      self.x.is_zero() | self.y.is_zero() | self.z.is_zero()
   }

   pub fn comp(&self) -> String {
      format!("vec3({}, {}, {})", self.x.comp(), self.y.comp(), self.z.comp())
   }

   pub fn new_from_f32(val: f32) -> Self {
      Self {
         x: Float::new(val),
         y: Float::new(val),
         z: Float::new(val),
      }
   }

   pub fn new(vals: [f32; 3]) -> Self {
      Self {
         x: Float::new(vals[0]),
         y: Float::new(vals[1]),
         z: Float::new(vals[2]),
      }
   }
}
impl Default for Vec3 {
   fn default() -> Self {
      Self {
         x: Default::default(),
         y: Default::default(),
         z: Default::default(),
      }
   }
}