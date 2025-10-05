use std::sync::Arc;
use std::thread;
use std::time::Duration;
use druid::{ExtEventSink, Target};
use im::Vector;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;
use crate::{HISTORY_SIZE, UPDATE_GPU};

#[derive(Clone, Debug)]
pub (crate) struct GPU {
    pub(crate)brand: String,
    pub(crate)name: String,
    pub(crate)temp_history: Vector<f64>,
    pub(crate)fan_speed_history: Vector<Vector<f64>>,
    pub(crate) used_mem_history: Vector<f64>,
    pub(crate)used_mem: f64,
    pub(crate)total_mem: f64,
}

impl GPU {
    pub (crate) fn new(sink: ExtEventSink) -> Self {
        let nvml = Nvml::init().unwrap();
        match GPU::handle_nvidia(sink, Arc::new(nvml)) {
            Ok(GPU) => {
                GPU
            }
            Err(err) => {
                panic!("{}", err);
            }
        }

    }

    pub (crate) fn handle_nvidia(sink: ExtEventSink, nvml: Arc<Nvml>) -> Result<GPU, NvmlError> {
        // Initialize histories with zeros; actual GPU data will be populated by the spawned thread.
        let mut temp_history = vec![0.0; HISTORY_SIZE];
        let mut fan_speed_history: Vector<Vector<f64>> = Vector::new();
        let mut used_mem_history = vec![0.0; HISTORY_SIZE];

        // Keep the Nvml alive inside the thread by moving the Arc into the closure.
        thread::spawn(move || {
            // Create the device inside the thread so its lifetime is tied to the Nvml kept here.
            let device = match nvml.device_by_index(0) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to get NVML device: {:?}", e);
                    return;
                }
            };

            let num_fans = match device.num_fans() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Failed to query number of fans: {:?}", e);
                    return;
                }
            };

            // Local working buffer for fan history accumulation
            let mut fan_history = vec![vec![0.0; HISTORY_SIZE]; num_fans as usize];

            loop { // Update temperature history
                if let Ok(temp) = device.temperature(TemperatureSensor::Gpu) {
                    temp_history.rotate_left(1);
                    temp_history[HISTORY_SIZE - 1] = temp as f64;
                }

                // Update fan histories
                for i in 0..num_fans as usize {
                    if let Ok(fan_speed) = device.fan_speed(i as u32) {
                        fan_history[i].rotate_left(1);
                        fan_history[i][HISTORY_SIZE - 1] = fan_speed as f64;
                    }
                }

                // convert Vec<Vec<f64>> -> im::Vector<im::Vector<f64>>
                for v in &fan_history {
                    fan_speed_history.push_back(Vector::from(v.clone()));
                }

                // Update memory info
                if let Ok(mem_info) = device.memory_info() {
                    used_mem_history.rotate_left(1);
                    used_mem_history[HISTORY_SIZE - 1] = mem_info.used as f64;

                    // Convert VRAM bytes history to percentage of total (0..100)
                    let total = if mem_info.total > 0 { mem_info.total } else { 1 };
                    let mut v: Vec<f64> = Vec::with_capacity(used_mem_history.len());

                    for val in used_mem_history.iter() {
                        let pct = (val / total as f64) * 100.0;
                        // clamp to [0, 100]
                        let pct = if pct.is_finite() { pct.max(0.0).min(100.0) } else { 0.0 };
                        v.push(pct);
                    }

                    let used_mem_history_percentage = Vector::from(v);

                    let brand = device
                        .brand()
                        .map(|b| format!("{:?}", b))
                        .unwrap_or_else(|_| "Unknown".to_string());
                    let name = device
                        .name()
                        .unwrap_or_else(|_| "Unknown".to_string());

                    let updated_gpu = GPU {
                        brand,
                        name,
                        temp_history: Vector::from(temp_history.clone()),
                        fan_speed_history: Vector::from(fan_speed_history.clone()),
                        used_mem_history: Vector::from(used_mem_history_percentage.clone()),
                        used_mem: mem_info.used as f64,
                        total_mem: mem_info.total as f64,
                    };

                    // Send update to UI
                    let _ = sink.submit_command(UPDATE_GPU, updated_gpu, Target::Auto);
                }

                thread::sleep(Duration::from_millis(200));
            }
        });

        Ok(GPU {
            brand: "unknown".to_string(),
            name: "unknown".to_string(),
            temp_history: Vector::from(vec![0.0; HISTORY_SIZE]),
            fan_speed_history: Vector::new(),
            used_mem_history: Vector::from(vec![0.0; HISTORY_SIZE]),
            used_mem: 0.0,
            total_mem: 0.0,
        })
    }
}
