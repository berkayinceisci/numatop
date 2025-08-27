use crate::numa_node::NumaNode;
use crate::proc_info::{
    ProcessInfo, RawCpuTimes, get_processes_with_cpu_affinity, parse_proc_stat_for_cores,
};
use crate::sys_numa_info::{get_all_present_cpu_indices, get_numa_node_data};
use cli_log::*;
use ratatui::layout::Rect;
use std::collections::HashMap;

#[derive(Debug)]
pub struct PopupState {
    pub show: bool,
    pub cpu_core_id: u32,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone)]
pub struct CpuCoreArea {
    pub cpu_id: u32,
    pub area: Rect,
}

pub struct App {
    pub numa_nodes: Vec<NumaNode>,
    pub prev_cpu_times: HashMap<u32, RawCpuTimes>,
    pub popup_state: PopupState,
    pub cpu_core_areas: Vec<CpuCoreArea>,
}

impl App {
    pub fn new() -> App {
        App {
            numa_nodes: vec![],
            prev_cpu_times: HashMap::new(),
            popup_state: PopupState {
                show: false,
                cpu_core_id: 0,
                processes: Vec::new(),
            },
            cpu_core_areas: Vec::new(),
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

    pub fn show_cpu_popup(&mut self, cpu_core_id: u32) {
        self.popup_state.show = true;
        self.popup_state.cpu_core_id = cpu_core_id;

        // Fetch processes with affinity to this CPU core
        match get_processes_with_cpu_affinity(cpu_core_id) {
            Ok(processes) => {
                self.popup_state.processes = processes;
            }
            Err(e) => {
                eprintln!("Error fetching processes for CPU {}: {}", cpu_core_id, e);
                self.popup_state.processes.clear();
            }
        }
    }

    pub fn hide_popup(&mut self) {
        self.popup_state.show = false;
        self.popup_state.processes.clear();
    }

    pub fn handle_mouse_click(&mut self, x: u16, y: u16) {
        // Check if the click falls within any CPU core area
        for core_area in &self.cpu_core_areas {
            if x >= core_area.area.x
                && x < core_area.area.x + core_area.area.width
                && y >= core_area.area.y
                && y < core_area.area.y + core_area.area.height
            {
                // Found a matching CPU core, show popup
                self.show_cpu_popup(core_area.cpu_id);
                return;
            }
        }
    }

    pub fn clear_cpu_core_areas(&mut self) {
        self.cpu_core_areas.clear();
    }

    pub fn add_cpu_core_area(&mut self, cpu_id: u32, area: Rect) {
        self.cpu_core_areas.push(CpuCoreArea { cpu_id, area });
    }
}
