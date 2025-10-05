mod ui;
mod widgets;
mod gpu;

use std::io::{Error, ErrorKind};
use druid::{AppLauncher, Data, Lens, LocalizedString, WindowDesc, LensExt, Selector, RenderContext, Target};
use sysinfo::{System, Cpu};
use std::thread;
use std::time::Duration;
use im::Vector;
use nvml_wrapper::Nvml;
use crate::gpu::GPU;

#[derive(Clone, Lens, Debug)]
struct State {
    cpu: CPU,
    gpu: GPU,
}

#[derive(Clone, Lens, Debug)]
struct CPU {
    cpu_history: Vector<Vector<f64>>,
    cpu_avg_history: Vector<f64>,
    used_mem_history: Vector<f64>,
    used_mem: f64,
    total_mem: f64,
}


impl Data for State {
    fn same(&self, _other: &Self) -> bool {
        // Always trigger a paint; avoids Data bound issues for f64
        false
    }
}

const HISTORY_SIZE: usize = 120; // number of samples per core
const UPDATE_METRICS: Selector<CPU> = Selector::new("update_metrics");
const UPDATE_GPU: Selector<GPU> = Selector::new("update_gpu");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sys = System::new_all();
    sys.refresh_cpu_all();

    let cores = sys.cpus().len();
    let mut initial_history = Vector::new();
    for _ in 0..cores {
        initial_history.push_back(Vector::from(vec![0.0; HISTORY_SIZE]));
    }

    let main_window = WindowDesc::new(ui::build_ui())
        .title(LocalizedString::new("Rust Druid System Monitor"))
        .window_size((800.0, 600.0));

    let launcher = AppLauncher::with_window(main_window);
    let sink = launcher.get_external_handle();

    // Set initial state
    let state = State {
        cpu: CPU {
            cpu_history: initial_history,
            cpu_avg_history: Vector::from(vec![0.0; HISTORY_SIZE]),
            used_mem_history: Vector::from(vec![0.0; HISTORY_SIZE]),
            used_mem: 0.0,
            total_mem: 0.0,
        },
        gpu: GPU::new(sink.clone())

    };

    thread::spawn(move || {
        let mut sys = System::new_all();
        let mut history = vec![vec![0.0; HISTORY_SIZE]; cores];
        let mut avg_history = vec![0.0; HISTORY_SIZE];
        let mut mem_history = vec![0.0; HISTORY_SIZE];

        loop {
            sys.refresh_cpu_all();
            sys.refresh_memory();

            let mut total_cpu_usage = 0.0;

            for (i, cpu) in sys.cpus().iter().enumerate() {
                history[i].rotate_left(1);
                history[i][HISTORY_SIZE - 1] = cpu.cpu_usage() as f64;
                total_cpu_usage += cpu.cpu_usage() as f64;
            }

            let mem_usage = (sys.used_memory() as f64 / sys.total_memory() as f64 ) * 100.0;
            mem_history.rotate_left(1);
            mem_history[HISTORY_SIZE - 1] = mem_usage.clone() as f64;

            let avg_cpu_usage = total_cpu_usage / cores as f64;
            avg_history.rotate_left(1);
            avg_history[HISTORY_SIZE - 1] = avg_cpu_usage.clone();

            let used_mem = sys.used_memory() as f64 / 1024.0 / 1024.0;
            let total_mem = sys.total_memory() as f64 / 1024.0 / 1024.0;

            // convert Vec<Vec<f64>> -> im::Vector<im::Vector<f64>>
            let mut history_vector = Vector::new();
            for v in &history {
                history_vector.push_back(Vector::from(v.clone()));
            }

            let updated_cpu = CPU {
                cpu_history: history_vector.clone(),
                cpu_avg_history: Vector::from(avg_history.clone()),
                used_mem_history: Vector::from(mem_history.clone()),
                used_mem,
                total_mem,
            };

            sink.submit_command(UPDATE_METRICS, updated_cpu, Target::Auto)
                .unwrap();

            thread::sleep(Duration::from_millis(200));
        }
    });



    launcher.launch(state).expect("Failed to launch app");
    Err(Box::new(Error::new(ErrorKind::Other, "Failed to launch app")))
}

