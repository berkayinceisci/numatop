use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;
use std::{fs, io};

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

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
}

fn get_process_info(pid: u32) -> io::Result<ProcessInfo> {
    // Read process name from /proc/PID/comm
    let comm_path = format!("/proc/{}/comm", pid);
    let name = fs::read_to_string(&comm_path)?.trim().to_string();

    Ok(ProcessInfo { pid, name })
}

pub fn get_processes_with_cpu_affinity(cpu_core_id: u32) -> io::Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();

    // Read /proc directory to get all process directories
    let proc_entries = fs::read_dir("/proc")?;

    for entry in proc_entries {
        let entry = entry?;
        let path = entry.path();

        // Check if directory name is a PID (numeric)
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            if let Ok(pid) = dir_name.parse::<u32>() {
                // Check if process has affinity to this CPU core
                if let Ok(has_affinity) = check_cpu_affinity(pid, cpu_core_id) {
                    if has_affinity {
                        if let Ok(process_info) = get_process_info(pid) {
                            processes.push(process_info);
                        }
                    }
                }
            }
        }
    }

    processes.sort_by(|a, b| b.pid.cmp(&a.pid));

    Ok(processes)
}

fn check_cpu_affinity(pid: u32, cpu_core_id: u32) -> io::Result<bool> {
    // Read CPU affinity from /proc/PID/status
    let status_path = format!("/proc/{}/status", pid);
    let status_content = fs::read_to_string(&status_path)?;

    // Look for the "Cpus_allowed_list" line
    for line in status_content.lines() {
        if line.starts_with("Cpus_allowed_list:") {
            let allowed_cpus = line.split(':').nth(1).unwrap_or("").trim();

            // Parse CPU list (can be ranges like "0-3,8-11" or individual "0,1,2")
            return Ok(parse_cpu_list(allowed_cpus, cpu_core_id));
        }
    }

    // If we can't find affinity info, assume it can run on this CPU
    Ok(true)
}

fn parse_cpu_list(cpu_list: &str, target_cpu: u32) -> bool {
    for part in cpu_list.split(',') {
        let part = part.trim();

        if part.contains('-') {
            // Handle ranges like "0-3"
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (
                    range_parts[0].trim().parse::<u32>(),
                    range_parts[1].trim().parse::<u32>(),
                ) {
                    if target_cpu >= start && target_cpu <= end {
                        return true;
                    }
                }
            }
        } else {
            // Handle individual CPU numbers
            if let Ok(cpu) = part.parse::<u32>() {
                if cpu == target_cpu {
                    return true;
                }
            }
        }
    }

    false
}
