use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

use eframe::{App, egui, Frame, Storage};
use eframe::egui::{Color32, Context, DragValue, TextStyle, Vec2};
use egui_node_graph2::{DataTypeTrait, Graph, GraphEditorState, InputId, InputParamKind, NodeDataTrait, NodeId, NodeResponse, NodeTemplateIter, NodeTemplateTrait, OutputId, UserResponseTrait, WidgetValueTrait};

#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct MyNodeData {
   template: NodeTypes,
}

#[derive(PartialEq, Eq, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum ConnectionTypes {
   Tree,
   Vec2,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum ValueTypes {
   Tree,
   Vec2 { value: Vec2 },
}
impl Default for ValueTypes {
   fn default() -> Self {
      Self::Vec2 {value: Vec2::ZERO }
   }
}
impl ValueTypes {
   pub fn try_to_vec2(self) -> anyhow::Result<Vec2> {
      if let ValueTypes::Vec2 { value } = self {
         Ok(value)
      } else {
         anyhow::bail!("Invalid cast from {:?} to vec2", self)
      }
   }
}


// Node Types
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeTypes {
   Main,
   Union,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyResponse {
   SetActiveNode(NodeId),
   ClearActiveNode,
}

#[derive(Default)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct MyGraphState {
   pub active_node: Option<NodeId>,
}
impl DataTypeTrait<MyGraphState> for ConnectionTypes {
   fn data_type_color(&self, _user_state: &mut MyGraphState) -> Color32 {
      match self {
         ConnectionTypes::Vec2 => Color32::from_rgb(238, 207, 109),
         ConnectionTypes::Tree => Color32::from_rgb(238, 207, 0),
      }
   }

   fn name(&self) -> Cow<'_, str> {
      match self {
         ConnectionTypes::Vec2 => Cow::Borrowed("2d vector"),
         ConnectionTypes::Tree => Cow::Borrowed("Tree connection"),
      }
   }
}




// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for NodeTypes {
   type NodeData = MyNodeData;
   type DataType = ConnectionTypes;
   type ValueType = ValueTypes;
   type UserState = MyGraphState;
   type CategoryType = &'static str;

   fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
      Cow::Borrowed(match self {
         NodeTypes::Main => "Add main",
         NodeTypes::Union => "New Union",
      })
   }

   // this is what allows the library to show collapsible lists in the node finder.
   fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
      match self {
         _ => vec!["Nodes"]
      }
   }

   fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
      // It's okay to delegate this to node_finder_label if you don't want to
      // show different names in the node finder and the node itself.
      self.node_finder_label(user_state).into()
   }

   fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
      MyNodeData { template: *self }
   }

   fn build_node(
      &self,
      graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
      _user_state: &mut Self::UserState,
      node_id: NodeId,
   ) {
      match self {

         NodeTypes::Main => {
            // main output
            graph.add_output_param(
               node_id,
               "Children".to_string(),
               ConnectionTypes::Tree,
            );
         }

         NodeTypes::Union => {
            // main input
            graph.add_input_param(
               node_id,
               "Parent".to_string(),
               ConnectionTypes::Tree,
               ValueTypes::Tree,
               InputParamKind::ConnectionOnly,
               true,
            );

            // test data
            graph.add_input_param(
               node_id,
               "Test".to_string(),
               ConnectionTypes::Vec2,
               ValueTypes::Vec2 {
                  value: egui::vec2(0.0, 0.0),
               },
               InputParamKind::ConstantOnly,
               true,
            );

            // main output
            graph.add_output_param(
               node_id,
               "Children".to_string(),
               ConnectionTypes::Tree,
            );

         }

      }
   }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
   type Item = NodeTypes;

   fn all_kinds(&self) -> Vec<Self::Item> {
      // This function must return a list of node kinds, which the node finder
      // will use to display it to the user. Crates like strum can reduce the
      // boilerplate in enumerating all variants of an enum.
      vec![
         NodeTypes::Main,
         NodeTypes::Union,
      ]
   }
}

impl WidgetValueTrait for ValueTypes {
   type Response = MyResponse;
   type UserState = MyGraphState;
   type NodeData = MyNodeData;
   fn value_widget(
      &mut self,
      param_name: &str,
      _node_id: NodeId,
      ui: &mut egui::Ui,
      _user_state: &mut MyGraphState,
      _node_data: &MyNodeData,
   ) -> Vec<MyResponse> {
      // This trait is used to tell the library which UI to display for the
      // inline parameter widgets.
      match self {

         ValueTypes::Tree => { ui.label("None value"); }

         ValueTypes::Vec2 { value } => {
            ui.label(param_name);
            ui.horizontal(|ui| {
               ui.label("x");
               ui.add(DragValue::new(&mut value.x));
               ui.label("y");
               ui.add(DragValue::new(&mut value.y));
            });
         }

      }
      // This allows you to return your responses from the inline widgets.
      Vec::new()
   }
}


impl UserResponseTrait for MyResponse {}
impl NodeDataTrait for MyNodeData {
   type Response = MyResponse;
   type UserState = MyGraphState;
   type DataType = ConnectionTypes;
   type ValueType = ValueTypes;

   fn bottom_ui(
      &self,
      ui: &mut egui::Ui,
      node_id: NodeId,
      graph: &Graph<MyNodeData, ConnectionTypes, ValueTypes>,
      user_state: &mut Self::UserState,
   ) -> Vec<NodeResponse<MyResponse, MyNodeData>>
   where
       MyResponse: UserResponseTrait,
   {

      let mut responses = vec![];
      let is_active = user_state
          .active_node
          .map(|id| id == node_id)
          .unwrap_or(false);


      ui.group(|ui| {
         ui.label(format!("{:?}", node_id));

         if let NodeTypes::Main = graph.nodes[node_id].user_data.template {
            if !is_active {
               if ui.button("👁 Set active").clicked() {
                  responses.push(NodeResponse::User(MyResponse::SetActiveNode(node_id)));
               }
            } else {
               let button =
                   egui::Button::new(egui::RichText::new("👁 Active").color(Color32::BLACK))
                       .fill(Color32::GOLD);
               if ui.add(button).clicked() {
                  responses.push(NodeResponse::User(MyResponse::ClearActiveNode));
               }
            }
         }
      });


      responses
   }
}


type MyGraph = Graph<MyNodeData, ConnectionTypes, ValueTypes>;
type MyEditorState = GraphEditorState<MyNodeData, ConnectionTypes, ValueTypes, NodeTypes, MyGraphState>;


#[derive(Default)]
pub struct NodeGraph {
   state: MyEditorState,

   graph_state: MyGraphState,
}

#[cfg(feature = "persistence")]
const PERSISTENCE_KEY: &str = "egui_node_graph";

#[cfg(feature = "persistence")]
impl NodeGraph {
   pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let state = cc
          .storage
          .and_then(|storage| eframe::get_value(storage, PERSISTENCE_KEY))
          .unwrap_or_default();
      Self {
         state,
         graph_state: MyGraphState::default(),
      }
   }

}

impl App for NodeGraph {
   fn update(&mut self, ctx: &Context, frame: &mut Frame) {
         egui::TopBottomPanel::top("top").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
               egui::widgets::global_dark_light_mode_switch(ui);
            });
         });
         let graph_response = egui::CentralPanel::default()
             .show(ctx, |ui| {
                self.state.draw_graph_editor(
                   ui,
                   AllMyNodeTemplates,
                   &mut self.graph_state,
                   Vec::default(),
                )
             })
             .inner;
         for node_response in graph_response.node_responses {
            // Here, we ignore all other graph events. But you may find
            // some use for them. For example, by playing a sound when a new
            // connection is created
            if let NodeResponse::User(user_event) = node_response {
               match user_event {
                  MyResponse::SetActiveNode(node) => self.graph_state.active_node = Some(node),
                  MyResponse::ClearActiveNode => self.graph_state.active_node = None,
               }
            }
         }

         let mut traverser = Traverser::new();

         if let Some(node) = self.graph_state.active_node {
            if self.state.graph.nodes.contains_key(node) {
               traverser.start_from(node, &mut self.state);
            } else {
               self.graph_state.active_node = None;
            }
         }
      }
}


struct Traverser<'a> {
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

   pub fn start_from(&mut self, start_node_id: NodeId, in_graph_state: &'a mut MyEditorState) {
      self.graph_state = Some(in_graph_state);
      println!("Traversing from {start_node_id:?}");


      // populate cash
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


      // Perform traversal logic here
      self.disclose_node(start_node_id);


      self.graph_state = None; // Clear the reference after use
   }

   fn disclose_node(&mut self, node_id: NodeId) {
      if let Some(graph_state) = &mut self.graph_state {
         let graph = &graph_state.graph;

         let node = &graph.nodes[node_id];

         let mut tree_children = find_tree_children_of_node(node_id, &self.inputs_cash, graph_state).unwrap_or(vec![]);

         let input_on_id = graph.nodes[tree_children[0].0].inputs[1].1;
         let input = &graph.inputs[input_on_id];
         println!("I1 {input:?}");


         println!("{tree_children:?}");
      }
   }
}

fn find_tree_children_of_node(
      node_id: NodeId, inputs_cash: &HashMap<(InputId),
      NodeId>, graph_state: &MyEditorState)
   -> Option<Vec<(NodeId, NodeTypes)>>
{
   let graph = &graph_state.graph;
   let node = &graph[node_id];

   let tree_children: Vec<(NodeId, NodeTypes)> =  node.outputs.iter().filter_map(|(data, output_id)| {
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




