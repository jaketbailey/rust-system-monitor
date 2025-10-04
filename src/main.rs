use druid::kurbo::BezPath;
use druid::widget::{Flex, Label};
use druid::{AppLauncher, Color, Data, Env, EventCtx, Lens, LocalizedString, PaintCtx, Widget, WidgetExt, WindowDesc, LensExt, Size, Event, LifeCycleCtx, LifeCycle, UpdateCtx, LayoutCtx, BoxConstraints, Selector};
use sysinfo::{System, Cpu};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Data, Lens)]
struct State {
    cpu_history: Vec<Vec<f64>>,
    used_mem: f64,
    total_mem: f64,
}

const HISTORY_SIZE: usize = 120; // number of samples per core
const UPDATE_METRICS: Selector<State> = Selector::new("update_metrics");

fn main() {

    // Set initial state
    let mut sys = System::new_all();
    sys.refresh_cpu_all();

    let cores = sys.cpus().len();
    let initial_history = vec![vec![0.0; HISTORY_SIZE]; cores];

    let state = State {
        cpu_history: initial_history,
        used_mem: 0.0,
        total_mem: 0.0,
    };

    let main_window = WindowDesc::new(build_ui(cores))
        .title(LocalizedString::new("Rust Druid CPU Monitor"))
        .window_size((800.0, 400.0));

    let launcher = AppLauncher::with_window(main_window);
    let sink = launcher.get_external_handle();

    thread::spawn(move || {
        let mut sys = System::new_all();
        let mut history = vec![vec![0.0; HISTORY_SIZE]; cores];

        loop {
            sys.refresh_cpu_all();
            sys.refresh_memory();

            for (i, cpu) in sys.cpus().iter().enumerate() {
                history[i].rotate_left(1);
                history[i][HISTORY_SIZE - 1] = cpu.cpu_usage() as f64;
            }

            let used_mem = sys.used_memory() as f64 / 1024.0 / 1024.0;
            let total_mem = sys.total_memory() as f64 / 1024.0 / 1024.0;

            let updated_state = State {
                cpu_history: history.clone(),
                used_mem,
                total_mem,
            };

            sink.submit_command(UPDATE_METRICS, updated_state, None)
                .unwrap();

            thread::sleep(Duration::from_millis(1000));
        }
    });

    launcher.launch(state).expect("Failed to launch app");
}

fn build_ui(cores: usize) -> impl Widget<State> {
    Flex::column()
        .with_child(Label::new(|data: &State, _env: &Env| {
            format!(
                "Memory: {:1}/{:1}" MB,
                data.used_mem / data.total_mem,
            )
        }))
        .with_spacer(10.0)
        .with_child(CpuGraph::new(cores).lens(State::cpu_history))
}

// Custom widget for per-core CPU graph
struct CpuGraph {
    cores: usize,
}

impl CpuGraph {
    fn new(cores: usize) -> Self {
        Self { cores }
    }
}

impl Widget<Vec<Vec<f64>>> for CpuGraph {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Vec<Vec<f64>>, env: &Env) {
        todo!()
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Vec<Vec<f64>>, env: &Env) {
        todo!()
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Vec<Vec<f64>>, data: &Vec<Vec<f64>>, env: &Env) {
        todo!()
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Vec<Vec<f64>>, env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Vec<Vec<f64>>, env: &Env) {
        todo!()
    }
}