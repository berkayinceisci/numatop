#[derive(Debug, Clone, Default)]
pub struct RawCpuTimes {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
}

impl RawCpuTimes {
    // Total time is the sum of all times
    pub fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }

    // Busy time is total time minus idle times (idle + iowait)
    pub fn busy(&self) -> u64 {
        self.user + self.nice + self.system + self.irq + self.softirq + self.steal
    }
}

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
