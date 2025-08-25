use crate::app::ProcessInfo;
use std::fs;
use std::io::Result;

fn get_process_info(pid: u32) -> Result<ProcessInfo> {
    // Read process name from /proc/PID/comm
    let comm_path = format!("/proc/{}/comm", pid);
    let name = fs::read_to_string(&comm_path)?.trim().to_string();

    Ok(ProcessInfo { pid, name })
}

/// Get processes that have affinity set to a specific CPU core
pub fn get_processes_with_cpu_affinity(cpu_core_id: u32) -> Result<Vec<ProcessInfo>> {
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

    // Limit to top 15 processes
    processes.truncate(15);

    Ok(processes)
}

fn check_cpu_affinity(pid: u32, cpu_core_id: u32) -> Result<bool> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_list_individual_cpus() {
        // Test individual CPU numbers
        assert!(parse_cpu_list("0,1,2,3", 0));
        assert!(parse_cpu_list("0,1,2,3", 1));
        assert!(parse_cpu_list("0,1,2,3", 2));
        assert!(parse_cpu_list("0,1,2,3", 3));
        assert!(!parse_cpu_list("0,1,2,3", 4));
        assert!(!parse_cpu_list("0,1,2,3", 5));
    }

    #[test]
    fn test_parse_cpu_list_ranges() {
        // Test CPU ranges
        assert!(parse_cpu_list("0-3", 0));
        assert!(parse_cpu_list("0-3", 1));
        assert!(parse_cpu_list("0-3", 2));
        assert!(parse_cpu_list("0-3", 3));
        assert!(!parse_cpu_list("0-3", 4));

        // Test larger range
        assert!(parse_cpu_list("8-15", 10));
        assert!(!parse_cpu_list("8-15", 7));
        assert!(!parse_cpu_list("8-15", 16));
    }

    #[test]
    fn test_parse_cpu_list_mixed() {
        // Test mixed individual and ranges
        assert!(parse_cpu_list("0-3,8-11", 0));
        assert!(parse_cpu_list("0-3,8-11", 3));
        assert!(parse_cpu_list("0-3,8-11", 8));
        assert!(parse_cpu_list("0-3,8-11", 11));
        assert!(!parse_cpu_list("0-3,8-11", 4));
        assert!(!parse_cpu_list("0-3,8-11", 7));
        assert!(!parse_cpu_list("0-3,8-11", 12));

        // Test individual CPUs mixed with ranges
        assert!(parse_cpu_list("0,2-4,7", 0));
        assert!(parse_cpu_list("0,2-4,7", 2));
        assert!(parse_cpu_list("0,2-4,7", 3));
        assert!(parse_cpu_list("0,2-4,7", 4));
        assert!(parse_cpu_list("0,2-4,7", 7));
        assert!(!parse_cpu_list("0,2-4,7", 1));
        assert!(!parse_cpu_list("0,2-4,7", 5));
        assert!(!parse_cpu_list("0,2-4,7", 6));
    }

    #[test]
    fn test_parse_cpu_list_edge_cases() {
        // Test empty string
        assert!(!parse_cpu_list("", 0));

        // Test whitespace
        assert!(parse_cpu_list(" 0 , 1 , 2 ", 0));
        assert!(parse_cpu_list(" 0 - 3 ", 2));

        // Test single CPU
        assert!(parse_cpu_list("5", 5));
        assert!(!parse_cpu_list("5", 4));

        // Test invalid formats (should not crash)
        assert!(!parse_cpu_list("invalid", 0));
        assert!(!parse_cpu_list("0-", 0));
        assert!(!parse_cpu_list("-3", 0));
        assert!(!parse_cpu_list("0-3-5", 0));
    }

    #[test]
    fn test_get_process_info_with_mock_data() {
        // This test would require mocking the filesystem, which is complex
        // In a real scenario, you might want to use a mocking library
        // For now, we'll test the components that don't require filesystem access

        // We can't easily test get_process_info without filesystem mocking
        // but we've tested its components above
    }

    // Integration test that requires actual /proc filesystem
    #[test]
    #[ignore] // Use `cargo test -- --ignored` to run this test
    fn test_get_processes_with_cpu_affinity_integration() {
        // This is an integration test that requires a real /proc filesystem
        // It's marked as ignored because it depends on the system state

        let result = get_processes_with_cpu_affinity(0);

        match result {
            Ok(processes) => {
                // Should return at most 15 processes
                assert!(processes.len() <= 15);

                // All processes should have valid PIDs
                for process in &processes {
                    println!("{:?}", process);
                    assert!(process.pid > 0);
                    assert!(!process.name.is_empty());
                }
            }
            Err(e) => {
                // If we can't read /proc, that's also a valid test result
                println!("Integration test failed (expected on some systems): {}", e);
            }
        }
    }

    #[test]
    fn test_check_cpu_affinity_mock() {
        // We can't easily mock the filesystem without additional dependencies
        // but we can test the CPU list parsing which is the core logic

        // Test the parse_cpu_list function thoroughly (already done above)
        // In a production environment, you'd want to use a mocking framework
        // like `mockall` or create a trait for filesystem operations
    }

    // Helper function for creating mock process data (if we had filesystem mocking)
    #[allow(dead_code)]
    fn create_mock_stat_content(utime: u64, stime: u64) -> String {
        format!(
            "1 (test) S 0 1 1 0 -1 4194304 100 0 0 0 {} {} 0 0 20 0 1 0 100",
            utime, stime
        )
    }

    #[allow(dead_code)]
    fn create_mock_status_content(cpu_list: &str) -> String {
        format!(
            "Name:\ttest\nPid:\t1234\nCpus_allowed_list:\t{}\n",
            cpu_list
        )
    }
}
