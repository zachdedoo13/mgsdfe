use std::borrow::Cow;
use std::fmt::Debug;

use eframe::egui;
use eframe::egui::{Color32, ComboBox, DragValue, Ui};
use egui_node_graph2::{DataTypeTrait, Graph, InputParamKind, NodeId, NodeTemplateTrait, WidgetValueTrait};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use shader_paser::{CombinationType, SdfType};
use shader_paser::datastructures::{Float, FloatOrOss, Vec3};

use crate::graph::{MyGraphState, MyNodeData, MyResponse};

/// self-explanatory
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, EnumIter)]
pub enum NodeTypes {
   /// main types
   Main,
   Union,
   Shape,

   /// abstractions
   Transform,
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
         NodeTypes::Transform => "New transform",
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
      let z_to_m = || {
         FW {
            data: Float::new(1.0),
            speed: 0.001,
            bounds: (0.0, f32::MAX),
         }
      };
      let _inv = || {
         FW {
            data: Float::new(1.0),
            speed: 0.001,
            bounds: (-f32::MAX, f32::MAX),
         }
      };


      let v3z_tm = || {
         V3W {
            data: Vec3::new_from_f32(1.0),
            speed: 0.001,
            bounds: (0.0, f32::MAX),
         }
      };
      let v3inv = || {
        V3W {
           data: Vec3::new_from_f32(0.0),
           speed: 0.001,
           bounds: (-f32::MAX, f32::MAX),
        }
      };

      let trans = || {
         ValueTypes::Transform {
            position: v3inv(),
            rotation: v3inv(),
            scale: z_to_m(),
         }
      };

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
               "tree_connection".to_string(),
               ConnectionTypes::Tree,
               ValueTypes::Tree,
               InputParamKind::ConnectionOnly,
               true,
            );

            // data
            graph.add_input_param(
               node_id,
               "union_type".to_string(),
               ConnectionTypes::None,
               ValueTypes::UnionType {
                  ty: CombinationType::Union,
                  strength: z_to_m(),
                  order: true,
               },
               InputParamKind::ConstantOnly,
               true,
            );

            graph.add_input_param(
               node_id,
               "transform".to_string(),
               ConnectionTypes::Transform,
               trans(),
               InputParamKind::ConnectionOrConstant,
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
               "tree_connection".to_string(),
               ConnectionTypes::Tree,
               ValueTypes::Tree,
               InputParamKind::ConnectionOnly,
               true,
            );

            graph.add_input_param(
               node_id,
               "sdf".to_string(),
               ConnectionTypes::None,
               ValueTypes::SdfData {
                  val: SdfType::Sphere,
                  data: v3z_tm(),
               },
               InputParamKind::ConstantOnly,
               true,
            );

            graph.add_input_param(
               node_id,
               "transform".to_string(),
               ConnectionTypes::Transform,
               trans(),
               InputParamKind::ConnectionOrConstant,
               true,
            );
         }

         NodeTypes::Transform => {
            graph.add_input_param(
               node_id,
               "transform".to_string(),
               ConnectionTypes::Transform,
               trans(),
               InputParamKind::ConstantOnly,
               true,
            );

            graph.add_output_param(
               node_id,
               "Doesn't work with\nmultiple connections".to_string(),
               ConnectionTypes::None,
            );

            graph.add_output_param(
               node_id,
               "Out".to_string(),
               ConnectionTypes::Transform,
            );
         }
      }
   }
}

/// ways in which a node can connect
#[derive(Serialize, Deserialize)]
#[derive(Eq, PartialOrd, PartialEq, Debug)]
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
         ConnectionTypes::None => Color32::TRANSPARENT,
         _ => Color32::GOLD,
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
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub enum ValueTypes {
   Tree,
   Float { val: f32 },

   UnionType { ty: CombinationType, strength: FW, order: bool },

   Transform { position: V3W, rotation: V3W, scale: FW },

   // Material { material: Material },

   SdfData { val: SdfType, data: V3W },

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

         ValueTypes::UnionType { ty, strength, order } => {
            ui.group(|ui| {
               combination_box(ty, ui);
               ui.horizontal(|ui| {
                  strength.ui(ui);

                  if ui.button(if *order { "<-" } else { "->" }).clicked() {
                     *order = !*order;
                  }
               });
            });
         }

         ValueTypes::Transform { position, rotation, scale } => {
            ui.group(|ui| {
               ui.group(|ui| {
                  ui.label("scale");
                  scale.ui(ui);
               });

               ui.group(|ui| {
                  ui.label("position");
                  position.ui(ui);
               });

               ui.group(|ui| {
                  ui.label("rotation");
                  rotation.ui(ui);
               });
            });
         }

         ValueTypes::SdfData { val, data } => {
            combination_box(val, ui);

            match val {
               SdfType::Cube => {
                  ui.group(|ui| {
                     ui.label("Dimensions");
                     data.ui(ui);
                  });
               }

               _ => {}
            }
         }

         // ValueTypes::Material { .. } => {}
      }
      // This allows you to return your responses from the inline widgets.
      Vec::new()
   }
}

/// float wrapper class
#[derive(Debug, Clone, Copy)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FW {
   pub data: Float,
   pub speed: f32,
   pub bounds: (f32, f32),
}
impl FW {
   pub fn ui(&mut self, ui: &mut Ui) {
      match &mut self.data.val {
         FloatOrOss::Float(val) => {
            ui.add(DragValue::new(val)
                .speed(self.speed)
                .range(self.bounds.0..=self.bounds.1));
         }
         FloatOrOss::Oss(_) => todo!()
      }
   }
}
impl Default for FW {
   fn default() -> Self {
      Self {
         data: Float::default(),
         speed: 0.01,
         bounds: (-f32::MAX, f32::MAX),
      }
   }
}

/// vec3 wrapper class
#[derive(Debug, Clone, Copy)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct V3W {
   pub data: Vec3,
   pub speed: f32,
   pub bounds: (f32, f32),
}
impl V3W {
   pub fn ui(&mut self, ui: &mut Ui) {
      ui.horizontal(|ui| {
         if let FloatOrOss::Float(val) = &mut self.data.x.val {
            ui.add(DragValue::new(val)
                      .speed(self.speed)
                      .range(self.bounds.0..=self.bounds.1));
         }

         if let FloatOrOss::Float(val) = &mut self.data.y.val {
            ui.add(DragValue::new(val)
                .speed(self.speed)
                .range(self.bounds.0..=self.bounds.1));
         }

         if let FloatOrOss::Float(val) = &mut self.data.z.val {
            ui.add(DragValue::new(val)
                .speed(self.speed)
                .range(self.bounds.0..=self.bounds.1));
         }
      });
   }
}
impl Default for V3W {
   fn default() -> Self {
      Self {
         data: Vec3::default(),
         speed: 0.01,
         bounds: (-f32::MAX, f32::MAX),
      }
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