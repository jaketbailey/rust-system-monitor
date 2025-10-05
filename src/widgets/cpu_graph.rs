use druid::{BoxConstraints, Color, Env, Event, EventCtx, FontFamily, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget};
use druid::kurbo::{BezPath, Line};
use druid::piet::{Text, TextLayoutBuilder};
use crate::{State, HISTORY_SIZE, UPDATE_METRICS};

const FONT_SIZE: f64 = 10.0;
const LABEL_COLOUR: Color = Color::grey8(220);

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

        let left_margin = 40.0;
        let bottom_margin = 30.0;
        let plot_rect = Rect::new(left_margin, 10.0, size.width - 10.0, size.height - bottom_margin);

        let colours = [
            Color::rgb8(0, 128, 255),
            Color::rgb8(0, 200, 128),
            Color::rgb8(255, 128, 0),
            Color::rgb8(200, 0, 128),
        ];

        // Draw background
        ctx.fill(plot_rect, &Color::grey8(20));

        // Draw axes
        let axis_color = Color::grey8(180);
        ctx.stroke(Line::new(
            (plot_rect.x0, plot_rect.y1),
            (plot_rect.x1, plot_rect.y1),
        ), &axis_color, 2.0); // X axis
        ctx.stroke(Line::new(
            (plot_rect.x0, plot_rect.y0),
            (plot_rect.x0, plot_rect.y1),
        ), &axis_color, 2.0); // Y axis

        // Draw y-axis tick labels
        for i in 0..=10 {
            let y = plot_rect.y1 - (i as f64) * (plot_rect.height() / 10.0);
            let label = format!("{}%", i * 10);
            ctx.stroke(Line::new(
                (plot_rect.x0 - 5.0, y),
                (plot_rect.x0, y),
            ), &axis_color, 1.0);

            let text = ctx.text()
                .new_text_layout(label)
                .text_color(LABEL_COLOUR)
                .font(FontFamily::SYSTEM_UI, FONT_SIZE)
                .build()
                .unwrap();

            ctx.draw_text(
                &text,
                (5.0, y - 6.0),
            );
        }

        for (i, core_history) in data.cpu_history.iter().enumerate() {
            let mut path = BezPath::new();
            let x_start = plot_rect.x0;
            let width = plot_rect.width();
            let height = plot_rect.height();
            let y_base = plot_rect.y1;
            let scale_x = width / (HISTORY_SIZE.saturating_sub(1) as f64);

            // Start at the left edge of the plotting area
            path.move_to((x_start, y_base - (core_history[0] / 100.0) * height));

            // Plot each point within the plotting rect, so it aligns with the axes
            for (x, &val) in core_history.iter().enumerate() {
                let x_pos = x_start + (x as f64) * scale_x;
                let y = y_base - (val / 100.0) * height;
                path.line_to((x_pos, y));
            }

            ctx.stroke(path.clone(), &colours[i % colours.len()], 2.0);

            // Fill down to the X axis within the plotting area
            let mut fill = path.clone();
            fill.line_to((plot_rect.x1, plot_rect.y1));
            fill.line_to((plot_rect.x0, plot_rect.y1));
            fill.close_path();
            ctx.fill(fill, &colours[i % colours.len()].with_alpha(0.15));
        }
    }
}