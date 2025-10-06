use druid::{BoxConstraints, Color, Env, Event, EventCtx, FontFamily, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget};
use druid::kurbo::{BezPath, Line};
use druid::piet::{Text, TextLayout, TextLayoutBuilder};
use im::Vector;
use crate::{State, HISTORY_SIZE, UPDATE_GPU, UPDATE_METRICS};
use crate::gpu::MAX_RPM;

const FONT_SIZE: f64 = 10.0;
const LABEL_COLOUR: Color = Color::grey8(220);

const COLOURS: [Color; 12] = [
    Color::rgb8(0, 128, 255),
    Color::rgb8(0, 200, 128),
    Color::rgb8(255, 128, 0),
    Color::rgb8(200, 0, 128),
    Color::rgb8(128, 0, 255),
    Color::rgb8(255, 0, 128),
    Color::rgb8(0, 255, 200),
    Color::rgb8(128, 255, 0),
    Color::rgb8(0, 255, 0),
    Color::rgb8(255, 0, 0),
    Color::rgb8(0, 128, 128),
    Color::rgb8(128, 128, 0),
];

#[derive(Clone, Copy)]
pub enum PlotType {
    AverageCPU,
    PerCoreCPU,
    RAM,
    GPU,
    GPUFan,
    GPUTemp
}

// Custom widget for per-core CPU graph
pub(crate) struct UsageGraph {
    plot_type: PlotType,
}

impl UsageGraph {
    pub(crate) fn new(plot_type: PlotType) -> Self {
        Self {
            plot_type,
        }
    }

    fn draw_line(ctx: &mut PaintCtx, plot_rect: Rect, color: &Color, history: Vector<f64>) {
        let mut path = BezPath::new();
        let x_start = plot_rect.x0;
        let width = plot_rect.width();
        let height = plot_rect.height();
        let y_base = plot_rect.y1;
        let scale_x = width / (HISTORY_SIZE.saturating_sub(1) as f64);

        // Start at the first history sample aligned to the left edge
        path.move_to((x_start, y_base - (history[0] / 100.0) * height));

        // Plot each subsequent point within the plotting rect, so it aligns with the axes
        for (x, &val) in history.iter().enumerate().skip(1) {
            let x_pos = x_start + (x as f64) * scale_x;
            let y = y_base - (val / 100.0) * height;
            path.line_to((x_pos, y));
        }

        ctx.stroke(path.clone(), color, 2.0);

        // Fill down to the X axis within the plotting area
        let mut fill = path.clone();
        fill.line_to((plot_rect.x1, plot_rect.y1));
        fill.line_to((plot_rect.x0, plot_rect.y1));
        fill.close_path();
        ctx.fill(fill, &color.with_alpha(0.15));
    }

    fn draw_legends(ctx: &mut PaintCtx, plot_rect: Rect, legend_x: f64, legend_y: f64, item_height: f64, text_offset: f64, items: &[(String, Color)]) {
        let mut x = legend_x;
        let mut y = legend_y;
        let right_limit = plot_rect.x1 - 10.0;
        let row_spacing = 4.0;
        let item_spacing_x = 16.0; // space after each item
        let sample_len = 8.0; // length of the legend line sample
        let text_baseline_offset = 6.0; // aligns with prior rendering

        for (label, colour) in items.iter() {
            // measure text
            let layout = ctx
                .text()
                .new_text_layout(label.clone())
                .text_color(Color::grey8(220))
                .font(FontFamily::SYSTEM_UI, 12.0)
                .build()
                .unwrap();
            let text_size = layout.size();

            // compute required width for this legend item
            let item_width = text_offset + text_size.width + item_spacing_x;

            // wrap if this item would overflow
            if x + item_width > right_limit {
                x = legend_x;
                y += item_height + row_spacing;
            }

            // draw sample line
            ctx.stroke(
                Line::new((x, y + text_baseline_offset), (x + sample_len, y + text_baseline_offset)),
                colour,
                3.0,
            );

            // draw text
            ctx.draw_text(&layout, (x + text_offset, y));

            // advance x for next item
            x += item_width;
        }
    }
}

impl Widget<State> for UsageGraph {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut State, env: &Env) {
        if let Event::Command(cmd) = event {
            if let Some(new_stats) = cmd.get(UPDATE_METRICS) {
                // Replace whole state (or selectively update fields)
                data.system.cpu_history = new_stats.cpu_history.clone();
                data.system.cpu_avg_history = new_stats.cpu_avg_history.clone();
                data.system.used_mem_history = new_stats.used_mem_history.clone();
                data.system.used_mem = new_stats.used_mem;
                data.system.total_mem = new_stats.total_mem;
                ctx.request_paint();
            } else if let Some(new_gpu) = cmd.get(UPDATE_GPU) {
                data.gpu = new_gpu.clone();
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

        for i in 0..=10 {
            let y = plot_rect.y1 - (i as f64) * (plot_rect.height() / 10.0);
            let label = match self.plot_type {
                PlotType::GPUFan => {
                    let step = MAX_RPM / 10; // 300
                    format!("{}RPM", i * step)
                }
                PlotType::GPUTemp => {
                    format!("{}°C", i * 10)
                }
                _ => {
                    format!("{}%", i * 10)
                }
            };
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

        let legend_x = plot_rect.x0 + 10.0;
        let legend_y = plot_rect.y0 + 10.0;
        let item_height = 16.0;
        let text_offset = 20.0;

        match self.plot_type {
            PlotType::AverageCPU => {
                UsageGraph::draw_line(ctx, plot_rect.clone(), &COLOURS[1], data.system.cpu_avg_history.clone());
            }
            PlotType::PerCoreCPU => {
                let mut items: Vec<(String, Color)> = Vec::new();
                for i in 0..data.system.cpu_history.len() {
                    let colour = COLOURS[i % COLOURS.len()].clone();
                    items.push((format!("Core {}", i + 1), colour));
                }
                UsageGraph::draw_legends(ctx, plot_rect, legend_x, legend_y, item_height, text_offset, &items);
                for (i, core_history) in data.system.cpu_history.iter().enumerate() {
                    let colour = &COLOURS[i % COLOURS.len()];
                    UsageGraph::draw_line(ctx, plot_rect.clone(), &colour, core_history.clone());
                }
            }
            PlotType::RAM => {
                UsageGraph::draw_line(ctx, plot_rect.clone(), &COLOURS[2], data.system.used_mem_history.clone());
            }
            PlotType::GPU => {
                UsageGraph::draw_line(ctx, plot_rect.clone(), &COLOURS[3], data.gpu.used_mem_history.clone())
            }
            PlotType::GPUFan => {
                // For each GPU fan, convert RPM history to percentage of 3000 RPM for plotting
                let mut items: Vec<(String, Color)> = Vec::new();
                for i in 0..data.gpu.fan_speed_history.len() {
                    let colour = COLOURS[i % COLOURS.len()].clone();
                    items.push((format!("Fan {}", i + 1), colour));
                }
                if !items.is_empty() {
                    UsageGraph::draw_legends(ctx, plot_rect, legend_x, legend_y, item_height, text_offset, &items);
                }
                for (i, fan_history) in data.gpu.fan_speed_history.iter().enumerate() {
                    let colour = &COLOURS[i % COLOURS.len()];
                    let mut v: Vec<f64> = Vec::with_capacity(fan_history.len());
                    for val in fan_history.iter() {
                        let pct = if MAX_RPM as f64 > 0.0 { (val / MAX_RPM as f64) * 100.0 } else { 0.0 };
                        let pct = if pct.is_finite() { pct.max(0.0).min(100.0) } else { 0.0 };
                        v.push(pct);
                    }
                    let pct_history = Vector::from(v);
                    UsageGraph::draw_line(ctx, plot_rect.clone(), &colour, pct_history);
                }
            }
            PlotType::GPUTemp => {
                // temp_history already stores temperatures in °C; draw_line expects values on 0..100 scale
                UsageGraph::draw_line(ctx, plot_rect.clone(), &COLOURS[4], data.gpu.temp_history.clone());
            }
        };

    }
}