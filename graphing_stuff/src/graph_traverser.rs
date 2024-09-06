use std::collections::HashMap;
use egui_node_graph2::{InputId, Node, NodeId, OutputId};

use common::{get, SHADER_GRAPH_DATA};
use shader_paser::{Combination, CombinationType, Layer, Passer, PassOptions, PassType, SDF};
use shader_paser::datastructures::{Float, FloatOrOss, Vec3};
use shader_paser::transform_mat::Transform;

use crate::graph::{MyEditorState, MyGraph, MyNodeData};
use crate::nodes_and_types::{ConnectionTypes, NodeTypes, ValueTypes};

type InOutCash = HashMap<OutputId, InputId>;
type OutputsCash = HashMap<OutputId, NodeId>;
type InputsCash = HashMap<InputId, NodeId>;

pub struct Traverser<'a> {
   graph_state: Option<&'a mut MyEditorState>,

   outputs_cash: OutputsCash,
   inputs_cash: InputsCash,
   out_to_in_cash: InOutCash,
}
impl<'a> Traverser<'a> {
   pub fn new() -> Self {
      Self {
         graph_state: None,
         outputs_cash: HashMap::new(),
         inputs_cash: HashMap::new(),
         out_to_in_cash: HashMap::new(),
      }
   }

   pub fn start_from(&mut self, start_node_id: NodeId, graph_state: &'a mut MyEditorState) {
      self.graph_state = Some(graph_state);

      if let Some(graph_state) = &mut self.graph_state {
         for (id, node) in graph_state.graph.nodes.iter() {
            for output in node.outputs.iter() {
               self.outputs_cash.insert(output.1, id);
            }

            for input in node.inputs.iter() {
               self.inputs_cash.insert(input.1, id);
            }
         }

         for (input, outputs) in graph_state.graph.connections.iter() {
            for output in outputs.iter() {
               self.out_to_in_cash.insert(*output, input);
            }
         }
      }

      // disclose nodes
      let out = self.disclose_node(start_node_id).expect("Failed to traverse tree");

      let mut passer = Passer {
         contents: &out,
         pass_options: PassOptions { pass_type: PassType::BruteForce },
      };


      let code = passer.pass();

      let sdg = &mut get!(SHADER_GRAPH_DATA);

      if code != sdg.shader_code.code {
         sdg.update_listener.queue_compile = true;
      }

      sdg.shader_code.code = code;


      self.graph_state = None;
   }

   fn disclose_node(&mut self, node_id: NodeId) -> Option<Layer> {
      if let Some(graph_state) = &mut self.graph_state {
         let graph = &graph_state.graph;
         let node = &graph.nodes[node_id];

         let layer = match &node.user_data.template {
            NodeTypes::Main => {
               let children_ids = find_tree_children_of_node(node_id, &self.inputs_cash, graph_state).unwrap();
               let children = children_ids.iter().filter_map(|&(id, _node_type)| {
                  self.disclose_node(id)
               }).collect();

               Layer::Union {
                  transform: Transform {
                     position: Vec3 {
                        x: Float { val: FloatOrOss::Float(0.0), id: 0},
                        y: Float { val: FloatOrOss::Float(0.0), id: 0},
                        z: Float { val: FloatOrOss::Float(0.0), id: 0},
                     },
                     rotation: Vec3 {
                        x: Float { val: FloatOrOss::Float(0.0), id: 0},
                        y: Float { val: FloatOrOss::Float(0.0), id: 0},
                        z: Float { val: FloatOrOss::Float(0.0), id: 0},
                     },
                     scale: Float { val: FloatOrOss::Float(1.0), id: 0},
                  },
                  bounds: Default::default(),
                  combination: Combination { comb: CombinationType::Union, strength: Default::default() },
                  children,
               }
            }

            NodeTypes::Union => {
               let trans_val = evaluate_connection(node, graph, &self.out_to_in_cash, &self.outputs_cash, "transform").unwrap();
               let transform = convert_transform(trans_val);

               let comb_val = evaluate_connection(node, graph, &self.out_to_in_cash, &self.outputs_cash, "union_type").unwrap();
               let combination = {
                  if let ValueTypes::UnionType { ty } = comb_val {
                     Combination {
                        comb: ty,
                        strength: Default::default(),
                     }
                  } else { panic!() }
               };

               let children_ids = find_tree_children_of_node(node_id, &self.inputs_cash, graph_state).unwrap();
               let children = children_ids.iter().filter_map(|&(id, _node_type)| {
                  self.disclose_node(id)
               }).collect();


               Layer::Union {
                  transform,
                  bounds: Default::default(),
                  combination,
                  children,
               }
            }

            NodeTypes::Shape => {
               let trans_val = evaluate_connection(node, graph, &self.out_to_in_cash, &self.outputs_cash, "transform").unwrap();
               let transform = convert_transform(trans_val);

               let sdf_val = evaluate_connection(node, graph, &self.out_to_in_cash, &self.outputs_cash, "sdf").unwrap();
               let sdf = {
                  if let ValueTypes::SdfData { val, data } = sdf_val {
                     SDF {
                        sdf_type: val,
                        settings: convert_vec3(data),
                     }
                  } else { panic!() }
               };


               Layer::Shape {
                  transform,
                  material: Default::default(),
                  bounds: Default::default(),
                  sdf,
               }
            }

            _ => { return None; }
         };

         return Some(layer);
      }

      None
   }
}

fn convert_transform(value_types: ValueTypes) -> Transform {
   if let ValueTypes::Transform { position, rotation, scale } = value_types {
      Transform {
         position: convert_vec3(position),
         rotation: convert_vec3(rotation),
         scale: convert_float(scale),
      }
   } else { panic!() }
}

fn convert_vec3(input: [f32; 3]) -> Vec3 {
   Vec3 {
      x: convert_float(input[0]),
      y: convert_float(input[1]),
      z: convert_float(input[2]),
   }
}

fn convert_float(input: f32) -> Float {
   Float {
      val: FloatOrOss::Float(input),
      id: 0,
   }
}

fn evaluate_connection<T: Into<String>>(
   node: &Node<MyNodeData>,
   graph: &MyGraph,
   out_to_in_cash: &InOutCash,
   outputs_cash: &OutputsCash,
   name: T,
) -> Option<ValueTypes> {
   let name = name.into();

   // Find the input in the current node
   let transform_input = node.inputs.iter().find(|param| param.0 == name)?.1;

   // Check if this input is connected to an output
   match out_to_in_cash.iter().find(|param| *param.1 == transform_input) {
      None => {
         // If not connected, get the value directly
         let transform_data = graph.inputs.get(transform_input)?;
         Some(transform_data.value.clone())
      }
      Some(connected_output) => {
         // If connected, follow the connection
         let connected_node_id = outputs_cash.get(connected_output.0)?;
         let connected_node = graph.nodes.get(*connected_node_id)?;

         // Find the corresponding input in the connected node
         let connected_transform_input = connected_node.inputs.iter().find(|param| param.0 == name)?.1;
         let transform_data = graph.inputs.get(connected_transform_input)?;

         Some(transform_data.value.clone())
      }
   }
}
fn find_tree_children_of_node(
   node_id: NodeId, inputs_cash: &InputsCash,
   graph_state: &MyEditorState) -> Option<Vec<(NodeId, NodeTypes)>>
{
   let graph = &graph_state.graph;
   let node = &graph[node_id];

   let tree_children: Vec<(NodeId, NodeTypes)> = node.outputs.iter().filter_map(|(_data, output_id)| {
      if let Some(out) = graph.outputs.get(*output_id) {
         if let ConnectionTypes::Tree = out.typ {
            // todo O(n^2)
            let child_nodes: Vec<(NodeId, NodeTypes)> = graph.connections.iter().filter_map(|(input_id, output_ids)| {
               if output_ids.contains(output_id) {
                  inputs_cash.get(&input_id).map(|&node_id| {
                     let node_type = graph.nodes[node_id].user_data.template;
                     (node_id, node_type)
                  })
               } else {
                  None
               }
            }).collect::<Vec<(NodeId, NodeTypes)>>();
            if !child_nodes.is_empty() {
               Some(child_nodes)
            } else {
               None
            }
         } else {
            None
         }
      } else {
         None
      }
   }).flatten().collect();

   return Some(tree_children);
}