use std::collections::HashMap;
use egui_node_graph2::{InputId, NodeId, OutputId, Node, Graph, InputParam};

use shader_paser::{Combination, CombinationType, Layer, Passer, PassOptions, PassType};
use shader_paser::transform_mat::Transform;
use crate::graph::{MyEditorState, MyGraph, MyNodeData};
use crate::nodes_and_types::{ConnectionTypes, NodeTypes, ValueTypes};

pub struct Traverser<'a> {
   graph_state: Option<&'a mut MyEditorState>,

   outputs_cash: HashMap<(OutputId), NodeId>,
   inputs_cash: HashMap<(InputId), NodeId>,
   out_to_in_cash: HashMap<OutputId, InputId>,

   depth: i32,
}
impl<'a> Traverser<'a> {
   pub fn new() -> Self {
      Self {
         graph_state: None,
         outputs_cash: HashMap::new(),
         inputs_cash: HashMap::new(),
         out_to_in_cash: HashMap::new(),
         depth: 0,
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


      /// disclose nodes
      let out = self.disclose_node(start_node_id).expect("Failed to traverse tree");


      println!("Out\n\n{out:?}\n\nend");

      let mut passer = Passer {
         contents: &out,
         pass_options: PassOptions { pass_type: PassType::BruteForce },
      };
      let code = passer.pass();

      // println!("Code\n\n{code}\n\nCode end\n");


      self.graph_state = None;
   }

   fn disclose_node(&mut self, node_id: NodeId) -> Option<Layer> {
      if let Some(graph_state) = &mut self.graph_state {
         let graph = &graph_state.graph;
         let node = &graph.nodes[node_id];

         let layer = match &node.user_data.template {
            NodeTypes::Main => {
               let children_ids = find_tree_children_of_node(node_id, &self.inputs_cash, graph_state).unwrap();
               let children = children_ids.iter().filter_map(|&(id, node_type)| {
                  self.disclose_node(id)
               }).collect();

               Layer::Union {
                  transform: Default::default(),
                  bounds: Default::default(),
                  combination: Combination {
                     comb: CombinationType::Union,
                     strength: Default::default(),
                  },
                  children,
               }
            }

            NodeTypes::Union => {
               todo!()
            }
            NodeTypes::Shape => {
               // let transform: Option<Transform> = {
               //    let transform_input = &node.inputs.iter().find(|param| param.0 == "transform").unwrap().1;
               //    let inter = match self.out_to_in_cash.iter().find(|param| param.1 == transform_input) /*todo shit ass slow un-cashed*/ {
               //       None => {
               //          let transform_data = graph.inputs.get(*transform_input).unwrap();
               //
               //          transform_data
               //       }
               //       Some(connected_output) => {
               //          let connected_node_id = self.outputs_cash.get(connected_output.0).unwrap();
               //          let connected_node = graph.nodes.get(*connected_node_id).unwrap();
               //
               //          let connected_transform_input = &connected_node.inputs.iter().find(|param| param.0 == "transform").unwrap().1;
               //          let transform_data = graph.inputs.get(*connected_transform_input).unwrap();
               //
               //          transform_data
               //       }
               //    };
               //
               //    None
               // };
               let transform = evaluate_connection(node, graph, &self.out_to_in_cash, &self.outputs_cash, "transform").unwrap();
               println!("{transform:?}");


               Layer::Shape {
                  transform: Default::default(),
                  material: Default::default(),
                  bounds: Default::default(),
                  sdf: Default::default(),
               }
            }

            _ => { return None; }
         };

         return Some(layer);
      }

      None
   }


}

fn evaluate_connection<T: Into<String>>(
   node: &Node<MyNodeData>,
   graph: &MyGraph,
   out_to_in_cash: &HashMap<OutputId, InputId>,
   outputs_cash: &HashMap<(OutputId), NodeId>,
   name: T
) -> Option<ValueTypes> {
   let name = name.into();

   let value: Option<ValueTypes> = {
      let transform_input = &node.inputs.iter().find(|param| param.0 == name)?.1;
      let inter =
          match out_to_in_cash.iter().find(|param| param.1 == transform_input) /*todo shit ass slow un-cashed*/ {
         None => {
            let transform_data = graph.inputs.get(*transform_input)?;

            Some(transform_data.value.clone())
         }
         Some(connected_output) => {
            let connected_node_id = outputs_cash.get(connected_output.0)?;
            let connected_node = graph.nodes.get(*connected_node_id)?;

            let connected_transform_input = &connected_node.inputs.iter().find(|param| param.0 == name)?.1;
            let transform_data = graph.inputs.get(*connected_transform_input)?;

            Some(transform_data.value.clone())
         }
      };

      inter
   };

   value
}
fn find_tree_children_of_node(
   node_id: NodeId, inputs_cash: &HashMap<(InputId),
      NodeId>, graph_state: &MyEditorState) -> Option<Vec<(NodeId, NodeTypes)>>
{
   let graph = &graph_state.graph;
   let node = &graph[node_id];

   let tree_children: Vec<(NodeId, NodeTypes)> = node.outputs.iter().filter_map(|(data, output_id)| {
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