use eframe::egui;
use eframe::egui::{Color32, Response, Ui};
use egui_node_graph2::{Graph, GraphEditorState, NodeDataTrait, NodeId, NodeResponse, NodeTemplateIter, UserResponseTrait};
use strum::IntoEnumIterator;

use crate::graph_traverser::Traverser;
use crate::nodes_and_types::*;

/// data held in each node
pub struct MyNodeData {
   pub template: NodeTypes,
}
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


/// internode interactivity
#[derive(Clone, Debug)]
pub enum MyResponse {
   SetActiveNode(NodeId),
   ClearActiveNode,
}
impl UserResponseTrait for MyResponse {}

/// passed to every node
#[derive(Default, Eq, PartialOrd, PartialEq)]
pub struct MyGraphState {
   pub active_node: Option<NodeId>,
}


pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
   type Item = NodeTypes;

   fn all_kinds(&self) -> Vec<Self::Item> {
      // This function must return a list of node kinds, which the node finder
      // will use to display it to the user. Crates like strum can reduce the
      // boilerplate in enumerating all variants of an enum.
      // vec![
      //    NodeTypes::Main,
      //    NodeTypes::Union,
      //    NodeTypes::Shape,
      // ]
      NodeTypes::iter().collect()
   }
}


// Graph code
pub type MyGraph = Graph<MyNodeData, ConnectionTypes, ValueTypes>;
pub type MyEditorState = GraphEditorState<MyNodeData, ConnectionTypes, ValueTypes, NodeTypes, MyGraphState>;

#[derive(Default)]
pub struct NodeGraph {
   state: MyEditorState,
   graph_state: MyGraphState,

   graph_on: bool,
}
impl NodeGraph {
   pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
      Self {
         state: MyEditorState::default(),
         graph_state: MyGraphState::default(),

         graph_on: true,
      }
   }

   pub fn update(&mut self, ui: &mut Ui) {
      egui::TopBottomPanel::top("options_thing").show_inside(ui, |ui| {
         toggle_ui_compact(ui, &mut self.graph_on);
      });

      egui::CentralPanel::default()
          .show_inside(ui, |ui| {
             ui.group(|ui| {
                if self.graph_on {
                   self.graph_ui(ui);
                }
             });
          })
          .inner;
   }

   fn graph_ui(&mut self, ui: &mut Ui) {
      let graph_response = self.state.draw_graph_editor(
         ui,
         AllMyNodeTemplates,
         &mut self.graph_state,
         Vec::default(),
      );

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


fn toggle_ui_compact(ui: &mut Ui, on: &mut bool) -> Response {
   let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
   let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
   if response.clicked() {
      *on = !*on;
      response.mark_changed();
   }
   response.widget_info(|| {
      egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
   });

   if ui.is_rect_visible(rect) {
      let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
      let visuals = ui.style().interact_selectable(&response, *on);
      let rect = rect.expand(visuals.expansion);
      let radius = 0.5 * rect.height();
      ui.painter()
          .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
      let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
      let center = egui::pos2(circle_x, rect.center().y);
      ui.painter()
          .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
   }

   response
}
