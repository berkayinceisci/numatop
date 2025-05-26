use crate::numa_node::RawCpuTimes;
use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;
use std::{fs, io};

pub fn parse_proc_stat_for_cores(
    cores_to_fetch: Vec<u32>,
) -> Result<HashMap<u32, RawCpuTimes>, Box<dyn Error>> {
    let mut all_core_times = HashMap::new();
    let file = fs::File::open("/proc/stat")?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("cpu") && !line.starts_with("cpu ") {
            let mut parts = line.split_whitespace();
            let cpu_label = parts.next().ok_or("Missing CPU label")?;
            if let Ok(core_id) = cpu_label[3..].parse::<u32>() {
                if cores_to_fetch.contains(&core_id) {
                    let times: Vec<u64> = parts.map(|s| s.parse().unwrap_or(0)).collect();
                    if times.len() >= 8 {
                        // user, nice, system, idle, iowait, irq, softirq, steal
                        all_core_times.insert(
                            core_id,
                            RawCpuTimes {
                                user: times[0],
                                nice: times[1],
                                system: times[2],
                                idle: times[3],
                                iowait: times[4],
                                irq: times[5],
                                softirq: times[6],
                                steal: times[7],
                            },
                        );
                    }
                }
            }
        }
    }
    Ok(all_core_times)
}
