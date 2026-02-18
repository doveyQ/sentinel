use sysinfo::{System, Disks, Networks, ProcessesToUpdate};

/// Information about a single process.
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub cmd: String,
}

/// Snapshot of all system statistics at a point in time.
pub struct SystemStats {
    pub hostname: String,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub disk_usage: Vec<(String, u64, u64)>,
    pub kernel: String,
    pub uptime: u64,
    pub processes: Vec<ProcessInfo>,
    pub network: Vec<(String, u64, u64)>,
    pub load_avg: (f64, f64, f64),
}

impl SystemStats {
    /// Refresh all data sources and return a new snapshot.
    pub fn collect(sys: &mut System, disks: &mut Disks, networks: &mut Networks) -> Self {
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        disks.refresh(true);
        networks.refresh(true);

        let disk_info: Vec<(String, u64, u64)> = disks
            .iter()
            .map(|d| {
                (
                    d.name().to_string_lossy().into_owned(),
                    d.total_space() - d.available_space(),
                    d.total_space(),
                )
            })
            .collect();

        let mut procs: Vec<ProcessInfo> = sys
            .processes()
            .iter()
            .map(|(pid, p)| {
                let cmd_str = p
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(" ");
                ProcessInfo {
                    pid: pid.as_u32(),
                    cpu_usage: p.cpu_usage(),
                    memory_mb: p.memory() as f64 / 1_048_576.0,
                    cmd: if cmd_str.is_empty() {
                        p.name().to_string_lossy().into_owned()
                    } else {
                        cmd_str
                    },
                }
            })
            .collect();
        procs.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        procs.truncate(10);

        let net_info: Vec<(String, u64, u64)> = networks
            .iter()
            .map(|(name, data)| {
                (
                    name.to_string(),
                    data.total_received(),
                    data.total_transmitted(),
                )
            })
            .collect();

        let load = System::load_average();

        SystemStats {
            hostname: System::host_name().unwrap_or_else(|| "unknown".into()),
            cpu_usage: sys.global_cpu_usage(),
            memory_used: sys.used_memory(),
            memory_total: sys.total_memory(),
            swap_used: sys.used_swap(),
            swap_total: sys.total_swap(),
            disk_usage: disk_info,
            kernel: System::kernel_version().unwrap_or_default(),
            uptime: System::uptime(),
            processes: procs,
            network: net_info,
            load_avg: (load.one, load.five, load.fifteen),
        }
    }

    /// Format uptime into a human-readable string.
    pub fn format_uptime(&self) -> String {
        let secs = self.uptime;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        if days > 0 {
            format!("{}d {}h {}m", days, hours, mins)
        } else if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m {}s", mins, secs % 60)
        }
    }

    /// Format bytes into human-readable units.
    pub fn format_bytes(bytes: u64) -> String {
        if bytes < 1_024 {
            format!("{} B", bytes)
        } else if bytes < 1_048_576 {
            format!("{:.1} KB", bytes as f64 / 1_024.0)
        } else if bytes < 1_073_741_824 {
            format!("{:.1} MB", bytes as f64 / 1_048_576.0)
        } else {
            format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
        }
    }
}