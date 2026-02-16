// src/stats.rs
use sysinfo::{System, Disks};
use rd_util::to_gb;

pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disk_usage: Vec<(String, u64, u64)>, 
    pub kernel: String,
    pub uptime: u64
}

impl SystemStats {
    pub fn collect(sys: &mut System, disks: &mut Disks) -> Self {
        sys.refresh_cpu_all();
        sys.refresh_memory();
        disks.refresh(true);
        
        let disk_info: Vec<(String, u64, u64)> = disks.iter().map(|disk| {
            (
                disk.name().to_string_lossy().into_owned(),
                disk.total_space() - disk.available_space(),
                disk.total_space(),
            )
        }).collect();

        SystemStats {
            cpu_usage: sys.global_cpu_usage(),
            memory_used: sys.used_memory(),
            memory_total: sys.total_memory(),
            disk_usage: disk_info,
            kernel: sysinfo::System::kernel_version().unwrap_or_default(),
            uptime: sysinfo::System::uptime()
        }
    }

    fn format_uptime(&self) -> String {
        if self.uptime < 60 {
            format!("{}s", self.uptime)
        } else {
            format!("{:.2} h", (self.uptime as f64 / 3600.0))
        }
    }
    
    pub fn display(&self) {
        println!("=== System Health Dashboard ===");
        println!("Kernel:       {}", self.kernel);
        println!("Uptime:       {}", self.format_uptime());
        println!("CPU Usage:    {:.2}%", self.cpu_usage);
        println!("Memory:       {:.2} / {:.2} GB", 
            to_gb(self.memory_used), 
            to_gb(self.memory_total)
        );

        println!("\nDisks:");
        for (name, used, total) in &self.disk_usage {
            println!("  {:<15} {:.2} / {:.2} GB", name, to_gb(*used), to_gb(*total));
        }
    }

}