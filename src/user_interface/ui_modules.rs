use std::fmt::Debug;
use egui::{ComboBox, Response, Ui};
use strum::IntoEnumIterator;

/// returns a true if changed
pub fn enum_combination_box<T, I>(ui: &mut Ui, combination_type: &mut T, label: I) -> bool
where
    T: IntoEnumIterator + Debug + PartialEq + Copy,
    I: Into<String>,
{
   let mut changed = false;
   ComboBox::from_label(label.into().as_str())
       .selected_text(format!("{combination_type:?}"))
       .show_ui(ui, |ui| {
          for variant in T::iter() {
             if ui.selectable_value(combination_type, variant, format!("{variant:?}")).changed() { changed = true; }
          }
       });

   changed
}

pub struct ToggleSwitch<'a> {
   on_off: &'a mut bool,
}

impl<'a> ToggleSwitch<'a> {
   pub fn new(val: &'a mut bool) -> Self {
      Self {
         on_off: val,
      }
   }
}

impl<'a> egui::Widget for ToggleSwitch<'a> {
   fn ui(self, ui: &mut Ui) -> Response {
      let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
      let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
      if response.clicked() {
         *self.on_off = !*self.on_off;
         response.mark_changed();
      }

      response.widget_info(|| {
         egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *self.on_off, "")
      });

      if ui.is_rect_visible(rect) {
         let how_on = ui.ctx().animate_bool_responsive(response.id, *self.on_off);
         let visuals = ui.style().interact_selectable(&response, *self.on_off);
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
}