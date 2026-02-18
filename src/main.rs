use std::thread;
use std::time::Duration;
use sysinfo::{System, Disks, Networks};

mod stats;
mod ui;

use stats::SystemStats;

fn main() {
    let mut sys: System = System::new_all();
    let mut disks: Disks = Disks::new_with_refreshed_list();
    let mut networks: Networks = Networks::new_with_refreshed_list();

    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        let stats = SystemStats::collect(&mut sys, &mut disks, &mut networks);

        println!("=== System Health Dashboard ===");
        println!("Hostname:     {}", stats.hostname);
        println!("Kernel:       {}", stats.kernel);
        println!("Uptime:       {}", stats.format_uptime());
        println!("Load Avg:     {:.2}  {:.2}  {:.2}  (1m / 5m / 15m)",
            stats.load_avg.0, stats.load_avg.1, stats.load_avg.2);
        println!("CPU Usage:    {:.2}%", stats.cpu_usage);
        println!("Memory:       {:.2} / {:.2} GB",
            rd_util::to_gb(stats.memory_used),
            rd_util::to_gb(stats.memory_total));
        println!("Swap:         {:.2} / {:.2} GB",
            rd_util::to_gb(stats.swap_used),
            rd_util::to_gb(stats.swap_total));

        println!("\nDisks:");
        for (name, used, total) in &stats.disk_usage {
            println!("  {:<15} {:.2} / {:.2} GB", name,
                rd_util::to_gb(*used), rd_util::to_gb(*total));
        }

        println!("\nNetwork:");
        for (iface, rx, tx) in &stats.network {
            println!("  {:<15} RX: {:>10}   TX: {:>10}",
                iface,
                SystemStats::format_bytes(*rx),
                SystemStats::format_bytes(*tx));
        }

        println!("\nTop Processes:");
        println!("  {:>7}  {:>6}  {:>8}  {}", "PID", "CPU%", "MEM(MB)", "COMMAND");
        println!("  {}", "-".repeat(60));
        for p in &stats.processes {
            let cmd_display = if p.cmd.len() > 40 {
                format!("{}…", &p.cmd[..39])
            } else {
                p.cmd.clone()
            };
            println!("  {:>7}  {:>5.1}%  {:>8.1}  {}", p.pid, p.cpu_usage, p.memory_mb, cmd_display);
        }

        thread::sleep(Duration::from_secs(2));
    }
}