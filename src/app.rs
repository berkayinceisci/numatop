use crate::numa_node::{CpuCore, NumaNode, RawCpuTimes};
use crate::proc_cpu_info::parse_proc_stat_for_cores;
use crate::sys_numa_info::{get_all_present_cpu_indices, get_numa_node_data};
use std::collections::HashMap;

pub struct App {
    pub numa_nodes: Vec<NumaNode>,
    pub prev_cpu_times: HashMap<u32, RawCpuTimes>,
}

impl App {
    pub fn new() -> App {
        App {
            numa_nodes: vec![],
            prev_cpu_times: HashMap::new(),
        }
    }

    pub fn update(&mut self) {
        // update numa node memory utilization
        match get_numa_node_data() {
            Ok(nodes) => self.numa_nodes = nodes,
            Err(e) => {
                eprintln!("Error fetching NUMA data: {}", e);
            }
        }

        // update core utilizations
        let current_raw_times =
            parse_proc_stat_for_cores(get_all_present_cpu_indices().unwrap()).unwrap();
        let mut current_cpu_utilizations = HashMap::new();
        for (pu_os_idx, current_times) in &current_raw_times {
            if let Some(prev_times) = self.prev_cpu_times.get(pu_os_idx) {
                let delta_total = current_times.total().saturating_sub(prev_times.total());
                let delta_busy = current_times.busy().saturating_sub(prev_times.busy());

                let utilization = if delta_total == 0 {
                    0.0
                } else {
                    (delta_busy as f64 / delta_total as f64) * 100.0
                };
                current_cpu_utilizations.insert(*pu_os_idx, utilization.min(100.0));
            } else {
                // First tick for this CPU, no prev data, so 0% utilization
                current_cpu_utilizations.insert(*pu_os_idx, 0.0);
            }
        }
        self.prev_cpu_times = current_raw_times;

        for node in &mut self.numa_nodes {
            if let Some(cpus) = &mut node.cpus {
                for cpu in cpus {
                    if let Some(utilization) = current_cpu_utilizations.get(&cpu.id) {
                        cpu.utilization = *utilization;
                    }
                }
            }
        }
    }
}
