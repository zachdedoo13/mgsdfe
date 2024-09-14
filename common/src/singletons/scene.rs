/// used to hold all data for the node-graph and raymarching
pub struct Scene {
   pub shapes: Vec<ShapeEntry>,

   pub cubemap: Vec<u8>,

   pub map_data: String,
}


pub struct ShapeEntry {
   pub name: String,
   pub shader_code: String,
}
impl ShapeEntry {

   /// pre-made shapes, called inside a switch case, has the inputs (vec3 p) and (vec3 data)
   pub fn hardcoded() -> Vec<ShapeEntry> {
      vec![
         // sphere
         ShapeEntry {
            name: "sphere".to_string(),
            shader_code: r#"

               return ShapeHit(length(p) - data.x);

            "#.to_string(),
         },
      ]
   }
}