use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, ErrorKind};
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

fn get_current_cpu_core(pid: u32) -> io::Result<u32> {
    // Construct the path to the process's stat file
    let stat_path = format!("/proc/{}/stat", pid);

    // Read the content of the stat file
    let stat_content = fs::read_to_string(&stat_path)?;

    // Split the content by spaces to get individual fields
    let fields: Vec<&str> = stat_content.split_whitespace().collect();

    // The 39th field (index 38) contains the current CPU core ID
    if let Some(cpu_field) = fields.get(38) {
        // Try to parse the field into a u32
        match cpu_field.parse::<u32>() {
            Ok(cpu_core) => Ok(cpu_core),
            Err(_) => Err(io::Error::new(
                ErrorKind::InvalidData,
                "Failed to parse CPU core ID",
            )),
        }
    } else {
        // If the 39th field doesn't exist, return an error
        Err(io::Error::new(
            ErrorKind::NotFound,
            "Could not find CPU core ID in /proc/PID/stat",
        ))
    }
}

pub fn get_processes_currently_on_core(cpu_core_id: u32) -> io::Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();

    // Read /proc directory to get all process directories
    let proc_entries = fs::read_dir("/proc")?;

    for entry in proc_entries {
        let entry = entry?;
        let path = entry.path();

        // Check if directory name is a PID (numeric)
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            if let Ok(pid) = dir_name.parse::<u32>() {
                // Check if the process is currently running on the specified CPU core
                if let Ok(current_cpu) = get_current_cpu_core(pid) {
                    if current_cpu == cpu_core_id {
                        // If it is, get the process info and add it to the list
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
