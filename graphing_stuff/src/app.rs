use std::{borrow::Cow, collections::HashMap};

use eframe::egui::{self, DragValue, TextStyle};
use egui_node_graph2::*;


/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub struct MyNodeData {
   template: MyNodeTemplate,
}


#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum MyDataType {
   TreeNode
}

#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum TreeNodeType {
   Main,
   Union,
   Shape,
}


#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum MyValueType {
   Placeholder { val: f32 },
}
impl Default for MyValueType {
   fn default() -> Self {
      // NOTE: This is just a dummy `Default` implementation. The library
      // requires it to circumvent some internal borrow checker issues.
      Self::Placeholder { val: 0.0 }
   }
}
impl MyValueType {
   /// Tries to downcast this value type to a vector
   pub fn try_to_placeholder(self) -> anyhow::Result<f32> {
      if let MyValueType::Placeholder { val } = self {
         Ok(val)
      } else {
         anyhow::bail!("Invalid cast from {:?} to placeholder", self)
      }
   }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum MyNodeTemplate {
   Main,
   Union,
   Shape,
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
impl DataTypeTrait<MyGraphState> for MyDataType {
   fn data_type_color(&self, _user_state: &mut MyGraphState) -> egui::Color32 {
      match self {
         MyDataType::TreeNode(_) => egui::Color32::from_rgb(38, 109, 211),
      }
   }

   fn name(&self) -> Cow<'_, str> {
      match self {
         MyDataType::TreeNode(_) => Cow::Borrowed("TreeNode"),
      }
   }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for MyNodeTemplate {
   type NodeData = MyNodeData;
   type DataType = MyDataType;
   type ValueType = MyValueType;
   type UserState = MyGraphState;
   type CategoryType = &'static str;

   fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
      Cow::Borrowed(match self {
         MyNodeTemplate::Main => "Main",
         MyNodeTemplate::Union => "Union",
         MyNodeTemplate::Shape => "Shape",
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

         MyNodeTemplate::Main => {
            graph.add_output_param(
               node_id,
               "Main out".to_string(),
               MyDataType::TreeNode,
            );
         }

         MyNodeTemplate::Union => {}
         MyNodeTemplate::Shape => {}
      }
   }
}