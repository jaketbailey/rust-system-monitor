mod ui;
mod gpu;
mod system;

use std::io::{Error, ErrorKind};
use druid::{AppLauncher, Data, Lens, LocalizedString, WindowDesc, LensExt, Selector, RenderContext};
use crate::gpu::GPU;
use crate::system::SystemStats;

#[derive(Clone, Lens, Debug)]
struct State {
    system: SystemStats,
    gpu: GPU,
}


impl Data for State {
    fn same(&self, _other: &Self) -> bool {
        // Always trigger a paint; avoids Data bound issues for f64
        false
    }
}

const HISTORY_SIZE: usize = 120; // number of samples per core
const UPDATE_METRICS: Selector<SystemStats> = Selector::new("update_metrics");
const UPDATE_GPU: Selector<GPU> = Selector::new("update_gpu");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = WindowDesc::new(ui::build_ui())
        .title(LocalizedString::new("Rust Druid System Monitor"))
        .window_size((800.0, 600.0));

    let launcher = AppLauncher::with_window(main_window);
    let sink = launcher.get_external_handle();


    let state = State {
        system: SystemStats::new(sink.clone()),
        gpu: GPU::new(sink.clone()),
    };

    launcher.launch(state).expect("Failed to launch app");
    Err(Box::new(Error::new(ErrorKind::Other, "Failed to launch app")))
}

