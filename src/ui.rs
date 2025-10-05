use druid::{ Env, Widget, WidgetExt};
use druid::widget::{CrossAxisAlignment, Flex, Label};
use crate::{State};
use crate::widgets::usage_graph::{UsageGraph, PlotType};

pub(crate) fn build_ui() -> impl Widget<State> {
    // Main content (existing column)
    let main_content = Flex::column()
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
                data.system.used_mem / 1024.0, data.system.total_mem / 1024.0
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
        .with_flex_child(UsageGraph::new(PlotType::GPU).expand_width(), 1.0);

    // Simple sidebar taking 1/5 of the width (flex 1 of total 5)
    let sidebar = Flex::column()
        .with_spacer(10.0)
        .with_child(Label::new("System Info").expand_width())
        .with_child(Label::new(""));

    // Combine sidebar and main content in a horizontal row with a 1:4 flex split
    Flex::row()
        .with_flex_child(sidebar, 1.0)
        .with_flex_child(main_content, 4.0)
        .cross_axis_alignment(CrossAxisAlignment::Start)
}