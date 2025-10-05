use druid::{ Env, Widget, WidgetExt};
use druid::widget::{Flex, Label};
use crate::{State};
use crate::widgets::usage_graph::{UsageGraph, PlotType};

pub(crate) fn build_ui() -> impl Widget<State> {
    Flex::column()
        // Average CPU Usage plot
        .with_child(Label::new(|_data: &State, _env: &Env| {
            "CPU Usage (average)".to_string()
        }))
        .with_flex_child(UsageGraph::new(PlotType::AverageCPU).expand_width(), 1.0)

        // CPU Core Usage plot
        .with_child(Label::new(|_data: &State, _env: &Env| {
            "CPU Core Usage".to_string()
        }))
        .with_flex_child(UsageGraph::new(PlotType::PerCoreCPU).expand_width(), 1.0)

        // RAM Usage plot
        .with_child(Label::new(|data: &State, _env: &Env| {
            format!(
                "RAM Usage: {:.2} GB / {:.2}GB ",
                data.used_mem / 1024.0, data.total_mem / 1024.0
            )
        }))
        .with_flex_child(UsageGraph::new(PlotType::RAM).expand_width(), 1.0)
}