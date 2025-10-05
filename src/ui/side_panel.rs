use druid::widget::{Flex, Label};
use druid::WidgetExt;
use crate::State;

pub (crate) fn side_panel() -> Flex<State> {
    // Simple sidebar taking 1/5 of the width (flex 1 of total 5)
    Flex::column()
        .with_spacer(10.0)
        .with_child(Label::new("System Info").expand_width())
        .with_child(Label::new(""))
}