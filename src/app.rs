use crate::numa_node::NumaNode;
use crate::sys_numa_info::get_numa_node_data;

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
