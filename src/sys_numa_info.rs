use crate::numa_node::{CpuCore, NumaNode};
use std::{
    collections::HashSet,
    error::Error,
    fs,
    io::{self, BufRead},
};

pub fn get_numa_node_data() -> Result<Vec<NumaNode>, Box<dyn Error>> {
    let mut nodes_info = Vec::new();
    let node_base_path = "/sys/devices/system/node";

    for entry in fs::read_dir(node_base_path)? {
        let path = entry?.path();

        if path.is_dir() {
            if let Some(name_osstr) = path.file_name() {
                let name = name_osstr.to_string_lossy();
                if name.starts_with("node") {
                    if let Ok(id) = name[4..].parse::<u32>() {
                        // Memory Info
                        let meminfo_path = path.join("meminfo");
                        let (total_mb, used_mb) =
                            parse_node_meminfo(&meminfo_path).unwrap_or_else(|e| {
                                eprintln!("Failed to parse meminfo for node {}: {}", id, e);
                                (0, 0)
                            });

                        // CPU Info
                        let cpulist_path = path.join("cpulist");
                        let mut node_cpus: Option<Vec<CpuCore>> = None;

                        if cpulist_path.exists() {
                            let cpulist_str = fs::read_to_string(cpulist_path)?;
                            if !cpulist_str.trim().is_empty() {
                                let core_ids = parse_cpulist(&cpulist_str);
                                if !core_ids.is_empty() {
                                    node_cpus = Some(
                                        core_ids
                                            .into_iter()
                                            .map(|core_id| CpuCore { id: core_id })
                                            .collect(),
                                    );
                                }
                            }
                        }

                        nodes_info.push(NumaNode {
                            id,
                            cpus: node_cpus,
                            total_memory_mb: total_mb,
                            used_memory_mb: used_mb,
                        });
                    }
                }
            }
        }
    }

    nodes_info.sort_by_key(|n| n.id);
    Ok(nodes_info)
}

fn parse_node_meminfo(path: &std::path::Path) -> Result<(u64, u64), Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut mem_total_kb: Option<u64> = None;
    let mut mem_free_kb: Option<u64> = None;
    let mut buffers_kb: Option<u64> = None;
    let mut cached_kb: Option<u64> = None;
    let mut slab_reclaimable_kb: Option<u64> = None;

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();
        let key = parts.next().unwrap_or("");
        let value_str = parts.next().unwrap_or("0");
        let value_kb = value_str.parse::<u64>().unwrap_or(0);

        match key {
            "MemTotal:" => mem_total_kb = Some(value_kb),
            "MemFree:" => mem_free_kb = Some(value_kb),
            "Buffers:" => buffers_kb = Some(value_kb),
            "Cached:" => cached_kb = Some(value_kb),
            "SReclaimable:" => slab_reclaimable_kb = Some(value_kb),
            _ => {}
        }
    }

    let total_kb = mem_total_kb.ok_or_else(|| format!("MemTotal not found in {:?}", path))?;
    let free_kb = mem_free_kb.ok_or_else(|| format!("MemFree not found in {:?}", path))?;
    let buffers_kb = buffers_kb.unwrap_or(0);
    let cached_kb = cached_kb.unwrap_or(0);
    let slab_reclaimable_kb = slab_reclaimable_kb.unwrap_or(0);

    let effective_free_kb = free_kb + buffers_kb + cached_kb + slab_reclaimable_kb;
    let used_kb = total_kb.saturating_sub(effective_free_kb);

    Ok((total_kb / 1024, used_kb / 1024)) // Convert KB to MB
}

// Basic parser for cpulist format like "0-3,7,10-11"
fn parse_cpulist(cpulist_str: &str) -> HashSet<u32> {
    let mut cpus = HashSet::new();
    for part in cpulist_str.trim().split(',') {
        if part.contains('-') {
            let range_parts: Vec<&str> = part.split('-').collect();
            if range_parts.len() == 2 {
                if let (Ok(start), Ok(end)) =
                    (range_parts[0].parse::<u32>(), range_parts[1].parse::<u32>())
                {
                    for cpu_id in start..=end {
                        cpus.insert(cpu_id);
                    }
                }
            }
        } else {
            if let Ok(cpu_id) = part.parse::<u32>() {
                cpus.insert(cpu_id);
            }
        }
    }
    cpus
}
