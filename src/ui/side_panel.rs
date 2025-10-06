use druid::widget::{Flex, Label};
use druid::{Env, WidgetExt};
use crate::State;

pub (crate) fn side_panel() -> Flex<State> {
    // Side panel
    Flex::column()
        .with_spacer(10.0)
        .with_child(Label::new("System Info").expand_width())
        .with_child(Label::new(|_data: &State, _env: &Env| {
            "CPU Usage (average)".to_string()
        }))
}