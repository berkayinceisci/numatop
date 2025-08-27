#[derive(Debug, Clone, Default)]
pub struct CpuCore {
    pub id: u32,
    pub utilization: f64,
}

#[derive(Debug, Clone)]
pub struct NumaNode {
    pub id: u32,
    pub cpus: Option<Vec<CpuCore>>, // None if CPULess, Some(vec![]) if has CPU region but no listed CPUs (unlikely for actual CPUs)
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
}
