use rand::Rng;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct CpuCore {
    pub id: u32,
    pub utilization: f32, // 0.0 to 100.0
}

#[derive(Debug, Clone)]
pub struct NumaNode {
    pub id: u32,
    pub cpus: Option<Vec<CpuCore>>,
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub has_cxl_expander: bool,
}

pub struct App {
    pub numa_nodes: Vec<NumaNode>,
}

impl App {
    pub fn new() -> App {
        App { numa_nodes: vec![] }
    }

    pub fn update(&mut self) {
        match get_numa_node_data() {
            Ok(nodes) => self.numa_nodes = nodes,
            Err(e) => {
                eprintln!("Error fetching NUMA data: {}", e);
            }
        }
    }
}

fn get_numa_node_data() -> Result<Vec<NumaNode>, Box<dyn Error>> {
    // Simulated data for demonstration
    Ok(vec![
        NumaNode {
            id: 0,
            cpus: Some(vec![
                CpuCore {
                    id: 0,
                    utilization: 15.5,
                },
                CpuCore {
                    id: 1,
                    utilization: 22.0,
                },
                CpuCore {
                    id: 2,
                    utilization: 8.7,
                },
                CpuCore {
                    id: 3,
                    utilization: 30.1,
                },
            ]),
            total_memory_mb: 16384,
            used_memory_mb: rand::rng().gen_range(0..16000),
            has_cxl_expander: false,
        },
        NumaNode {
            id: 1,
            cpus: Some(vec![
                CpuCore {
                    id: 4,
                    utilization: 10.0,
                },
                CpuCore {
                    id: 5,
                    utilization: 12.5,
                },
                CpuCore {
                    id: 6,
                    utilization: 18.3,
                },
                CpuCore {
                    id: 7,
                    utilization: 5.5,
                },
            ]),
            total_memory_mb: 16384,
            used_memory_mb: 8192,
            has_cxl_expander: false,
        },
        NumaNode {
            id: 2,
            cpus: None,
            total_memory_mb: 32768,
            used_memory_mb: 1024,
            has_cxl_expander: true,
        },
    ])
}
