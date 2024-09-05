use std::borrow::Cow;
use std::fmt::{Debug, format};
use eframe::egui;
use eframe::egui::{Color32, ComboBox, DragValue, Ui};
use egui_node_graph2::{DataTypeTrait, Graph, InputParamKind, NodeId, NodeTemplateTrait, WidgetValueTrait};
use strum::{EnumIter, IntoEnumIterator};

use shader_paser::{CombinationType, SdfType};

use crate::graph::{MyGraphState, MyNodeData, MyResponse};

/// self-explanatory
#[derive(Copy, Clone)]
pub enum NodeTypes {
   Main,
   Union,

   Shape,
}

/// A trait for the node kinds, which tells the library how to build new nodes
/// from the templates in the node finder
impl NodeTemplateTrait for NodeTypes {
   type NodeData = MyNodeData;
   type DataType = ConnectionTypes;
   type ValueType = ValueTypes;
   type UserState = MyGraphState;
   type CategoryType = &'static str;

   fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
      Cow::Borrowed(match self {
         NodeTypes::Main => "Main",
         NodeTypes::Union => "New Union",
         NodeTypes::Shape => "New Shape",
         _ => "New x"
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
               "".to_string(),
               ConnectionTypes::Tree,
               ValueTypes::Tree,
               InputParamKind::ConnectionOnly,
               true,
            );

            // data
            graph.add_input_param(
               node_id,
               "".to_string(),
               ConnectionTypes::None,
               ValueTypes::UnionType { val: CombinationType::Union },
               InputParamKind::ConstantOnly,
               true,
            );

            graph.add_input_param(
               node_id,
               "".to_string(),
               ConnectionTypes::Transform,
               ValueTypes::Transform {
                  position: [0.0, 0.0, 0.0],
                  rotation: [0.0, 0.0, 0.0],
                  scale: 0.0,
               },
               InputParamKind::ConnectionOnly,
               true,
            );


            // main output
            graph.add_output_param(
               node_id,
               "Children".to_string(),
               ConnectionTypes::Tree,
            );
         }
         NodeTypes::Shape => {
            graph.add_input_param(
               node_id,
               "".to_string(),
               ConnectionTypes::Tree,
               ValueTypes::Tree,
               InputParamKind::ConnectionOnly,
               true,
            );

            graph.add_input_param(
               node_id,
               "".to_string(),
               ConnectionTypes::None,
               ValueTypes::SdfData {
                  val: SdfType::Sphere,
                  data: [0.0, 0.0, 0.0],
               },
               InputParamKind::ConstantOnly,
               true,
            );

            graph.add_input_param(
               node_id,
               "".to_string(),
               ConnectionTypes::Transform,
               ValueTypes::Transform {
                  position: [0.0, 0.0, 0.0],
                  rotation: [0.0, 0.0, 0.0],
                  scale: 0.0,
               },
               InputParamKind::ConnectionOnly,
               true,
            );
         }
      }
   }
}

/// ways in which a node can connect
#[derive(Eq, PartialOrd, PartialEq)]
pub enum ConnectionTypes {
   /// used for all compiled stuff
   Tree,
   Transform,

   /// other data types
   Float,
   Vec3,

   None,
}
impl DataTypeTrait<MyGraphState> for ConnectionTypes {
   fn data_type_color(&self, _user_state: &mut MyGraphState) -> Color32 {
      match self {
         ConnectionTypes::Float => Color32::RED,
         ConnectionTypes::Tree => Color32::GOLD,
         ConnectionTypes::Vec3 => Color32::GOLD,
         ConnectionTypes::None => Color32::TRANSPARENT,
         ConnectionTypes::Transform => Color32::GRAY,
      }
   }

   fn name(&self) -> Cow<'_, str> {
      Cow::Borrowed(match self {
         ConnectionTypes::Float => "Floating point value",
         ConnectionTypes::Tree => "Tree connection",
         ConnectionTypes::Vec3 => "Vector 3",
         ConnectionTypes::None => "None",
         ConnectionTypes::Transform => "Transform"
      })
   }
}



/// data held by connections
#[derive(Debug)]
pub enum ValueTypes {
   Tree,
   Float { val: f32 },

   UnionType { val: CombinationType },

   Transform { position: [f32; 3], rotation: [f32; 3], scale: f32 },

   SdfData { val: SdfType, data: [f32; 3] },

   None,
}
impl Default for ValueTypes {
   fn default() -> Self {
      Self::None
   }
}
impl ValueTypes {
   pub fn try_to_none(self) -> anyhow::Result<i32> {
      if let ValueTypes::None = self { Ok(0) } else {
         anyhow::bail!("Invalid cast from {:?} to none", self)
      }
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
         ValueTypes::Tree => { ui.label("Tree connection"); }

         ValueTypes::Float { val } => {
            ui.label(param_name);
            ui.horizontal(|ui| {
               ui.label("val");
               ui.add(DragValue::new(val));
            });
         }

         ValueTypes::None => {}

         ValueTypes::UnionType { val } => {
            combination_box(val, ui);
         }

         ValueTypes::Transform { position, rotation, scale } => {
            ui.group(|ui| {
               ui.label("scale");
               ui.horizontal(|ui| {
                  ui.add(DragValue::new(scale).speed(0.01));
               });
            });

            ui.group(|ui| {
               ui.label("position");
               ui.horizontal(|ui| {
                  ui.add(DragValue::new(&mut position[0]).speed(0.01));
                  ui.add(DragValue::new(&mut position[1]).speed(0.01));
                  ui.add(DragValue::new(&mut position[2]).speed(0.01));
               });
            });

            ui.group(|ui| {
               ui.label("rotation");
               ui.horizontal(|ui| {
                  ui.add(DragValue::new(&mut rotation[0]).speed(0.01));
                  ui.add(DragValue::new(&mut rotation[1]).speed(0.01));
                  ui.add(DragValue::new(&mut rotation[2]).speed(0.01));
               });
            });
         }

         ValueTypes::SdfData { val, data } => {
            combination_box(val, ui);

            match val {
               SdfType::Cube => {
                  ui.group(|ui| {
                     ui.label("Dimensions");
                     ui.horizontal(|ui| {
                        ui.add(DragValue::new(&mut data[0]).speed(0.01));
                        ui.add(DragValue::new(&mut data[1]).speed(0.01));
                        ui.add(DragValue::new(&mut data[2]).speed(0.01));
                     });
                  });
               }

               _ => {}
            }
         }
      }
      // This allows you to return your responses from the inline widgets.
      Vec::new()
   }
}



pub fn combination_box<T: IntoEnumIterator + Debug + PartialEq + Copy>(combination_type: &mut T, ui: &mut Ui) {
   ComboBox::from_label("Combination Type")
       .selected_text(format!("{combination_type:?}"))
       .show_ui(ui, |ui| {
          for variant in T::iter() {
             ui.selectable_value(combination_type, variant, format!("{variant:?}"));
          }
       });
}