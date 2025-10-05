mod side_panel;
mod main_panel;
pub(crate) mod usage_graph;

use druid::{Widget, WidgetExt};
use druid::widget::{CrossAxisAlignment, Flex};
use crate::State;
use crate::ui::main_panel::main_panel;
use crate::ui::side_panel::side_panel;

pub(crate) fn build_ui() -> impl Widget<State> {
    // Combine sidebar and main content in a horizontal row with a 1:4 flex split
    Flex::row()
        .with_flex_child(side_panel(), 1.0)
        .with_flex_child(main_panel(), 4.0)
        .cross_axis_alignment(CrossAxisAlignment::Start)
}