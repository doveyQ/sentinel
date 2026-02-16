use std::thread;
use std::time::Duration;
use sysinfo::{System, Disks};

mod stats;

use stats::SystemStats;


fn main() {
    let mut sys: System = System::new_all();
    let mut disk: Disks = Disks::new_with_refreshed_list();
    
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        
        let stats: SystemStats = SystemStats::collect(&mut sys, &mut disk);
        stats.display();
        
        thread::sleep(Duration::from_secs(2));
    }
}