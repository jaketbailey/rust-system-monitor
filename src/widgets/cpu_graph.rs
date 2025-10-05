use druid::{BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, RenderContext, Size, UpdateCtx, Widget};
use druid::kurbo::BezPath;
use crate::{State, HISTORY_SIZE, UPDATE_METRICS};

// Custom widget for per-core CPU graph
pub (crate) struct CpuGraph {}

impl CpuGraph {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl Widget<State> for CpuGraph {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        if let Event::Command(cmd) = event {
            if let Some(new_state) = cmd.get(UPDATE_METRICS) {
                // Replace whole state (or selectively update fields)
                data.cpu_history = new_state.cpu_history.clone();
                data.used_mem = new_state.used_mem;
                data.total_mem = new_state.total_mem;
                ctx.request_paint();
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &State, env: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &State, data: &State, env: &Env) {
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &State, env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &State, env: &Env) {
        let size = ctx.size();

        let colours = [
            Color::rgb8(0, 128, 255),
            Color::rgb8(0, 200, 128),
            Color::rgb8(255, 128, 0),
            Color::rgb8(200, 0, 128),
        ];

        for (i, core_history) in data.cpu_history.iter().enumerate() {
            let mut path = BezPath::new();
            path.move_to((0.0, size.height - (core_history[0] / 100.0) * size.height));
            for (x, &val) in core_history.iter().enumerate() {
                let y = size.height - (val / 100.0) * size.height;
                path.line_to((x as f64 * size.width / HISTORY_SIZE as f64, y));
            }

            ctx.stroke(path.clone(), &colours[i % colours.len()], 2.0);

            let mut fill = path.clone();
            fill.line_to((size.width, size.height));
            fill.line_to((0.0, size.height));
            fill.close_path();
            ctx.fill(fill, &colours[i % colours.len()].with_alpha(0.15));
        }
    }
}