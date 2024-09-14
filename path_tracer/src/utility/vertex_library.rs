use crate::utility::vertex_package::Vertex;

pub const SQUARE_VERTICES: &[Vertex] = &[
   Vertex { position: [1.0, 1.0, 0.0] }, // Vertex 0: top-right
   Vertex { position: [-1.0, -1.0, 0.0] }, // Vertex 1: bottom-left
   Vertex { position: [1.0, -1.0, 0.0] }, // Vertex 2: bottom-right
   Vertex { position: [-1.0, 1.0, 0.0] }, // Vertex 3: top-left
];
pub const SQUARE_INDICES: &[u16] = &[
   0, 1, 2,
   3, 1, 0,
];