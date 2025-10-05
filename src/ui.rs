use druid::{ Env, Widget, WidgetExt};
use druid::widget::{Flex, Label};
use crate::{State};
use crate::widgets::usage_graph::{UsageGraph, PlotType};

pub(crate) fn build_ui() -> impl Widget<State> {
    Flex::column()
        .with_spacer(10.0)
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
                data.cpu.used_mem / 1024.0, data.cpu.total_mem / 1024.0
            )
        }))
        .with_flex_child(UsageGraph::new(PlotType::RAM).expand_width(), 1.0)

        .with_child(Label::new(|data: &State, _env: &Env| {
            format!(
                "GPU: {} \nVRAM Usage: {:.2} GB / {:.2}GB ",
                data.gpu.name,
                data.gpu.used_mem / 1024.0 / 1024.0 / 1024.0,
                data.gpu.total_mem / 1024.0 / 1024.0 / 1024.0
            )
        }))
        .with_flex_child(UsageGraph::new(PlotType::GPU).expand_width(), 1.0)

}