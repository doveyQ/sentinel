# Sentinel
Simple cli-based system monitor using sysinfo for data collection and ratatui for the user interface.   
Mainly for learning purposes! 

## Features
Usual stuff like:
- System informations
- Resources
- Top processes
- Disks & Network interfaces

## Installation (using Rust)

**Global Install** (Recommended):
```bash
cargo install --git https://github.com/doveyQ/sentinel.git
```

**Local Build** (For development):
```bash
git clone https://github.com/doveyQ/sentinel.git
cd sentinel && cargo run --release
```

## Usage
- Press `q` or `Esc` to quit.
- Data refreshes every 2 seconds.

> Used Opus 4.6 to implement the initial version of the UI.
