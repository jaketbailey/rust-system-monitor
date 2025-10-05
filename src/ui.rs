use druid::{ Env, Widget, WidgetExt};
use druid::widget::{Flex, Label};
use crate::{State};
use crate::widgets::cpu_graph::CpuGraph;

pub(crate) fn build_ui(cores: usize) -> impl Widget<State> {
    Flex::column()
        .with_child(Label::new(|data: &State, _env: &Env| {
            format!(
                "Memory: {:1}/{:1} MB",
                data.used_mem, data.total_mem,
            )
        }))
        .with_spacer(10.0)
        .with_flex_child(CpuGraph::new().expand_width(),1.0)
}
