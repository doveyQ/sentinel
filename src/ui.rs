use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table},
};
use rd_util::to_gb;

use crate::stats::SystemStats;


const ACCENT: Color = Color::Rgb(100, 180, 220);
const DIM: Color = Color::Rgb(90, 90, 100);
const TEXT: Color = Color::Rgb(200, 200, 210);
const GOOD: Color = Color::Rgb(80, 190, 120);
const WARN: Color = Color::Rgb(220, 180, 70);
const CRIT: Color = Color::Rgb(210, 90, 90);
const BAR_BG: Color = Color::Rgb(35, 35, 45);
const ROW_ALT: Color = Color::Rgb(25, 25, 35);

/// Returns the color for a given percentage.
fn usage_color(pct: f64) -> Color {
    match pct {
        p if p < 60.0 => GOOD,
        p if p < 85.0 => WARN,
        _ => CRIT,
    }
}

/// Percentage from a used/total pair, safe against division by zero.
fn pct(used: u64, total: u64) -> f64 {
    if total == 0 { 0.0 } else { (used as f64 / total as f64) * 100.0 }
}

/// Returns a styled block with the given title.
fn styled_block(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
}

/// One line of "label  value" for the system-info panel.
fn info_line<'a>(label: &'a str, value: impl Into<String>) -> Line<'a> {
    Line::from(vec![
        Span::styled(label, Style::default().fg(DIM)),
        Span::styled(value.into(), Style::default().fg(TEXT)),
    ])
}

/// A header cell with accent styling.
fn hdr(text: &str) -> Cell<'_> {
    Cell::from(text).style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
}

pub fn draw(frame: &mut Frame, stats: &SystemStats) {
    let [_, top, mid, bot, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(8),
        Constraint::Min(8),
        Constraint::Length(9),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_top_row(frame, stats, top);
    draw_process_table(frame, stats, mid);
    draw_bottom_row(frame, stats, bot);
    draw_footer(frame, footer);
}


fn draw_footer(frame: &mut Frame, area: Rect) {
    let footer = Line::from(vec![
        Span::styled(" q", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
        Span::styled(" quit  ", Style::default().fg(DIM)),
        Span::styled("refresh: ", Style::default().fg(DIM)),
        Span::styled("2s", Style::default().fg(TEXT)),
    ]);
    frame.render_widget(Paragraph::new(footer), area);
}


fn draw_top_row(frame: &mut Frame, stats: &SystemStats, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .areas(area);

    // ── system info ──
    let info = Paragraph::new(vec![
        info_line(" Hostname  ", &stats.hostname),
        info_line(" Kernel    ", &stats.kernel),
        info_line(" Uptime    ", stats.format_uptime()),
        info_line(
            " Load Avg  ",
            format!("{:.2}  {:.2}  {:.2}", stats.load_avg.0, stats.load_avg.1, stats.load_avg.2),
        ),
        info_line("           ", "1m    5m    15m").style(
            Style::default().fg(DIM).add_modifier(Modifier::ITALIC),
        ),
    ])
    .block(styled_block("System"));
    frame.render_widget(info, left);

    // ── resource gauges ──
    let block = styled_block("Resources");
    let inner = block.inner(right);
    frame.render_widget(block, right);

    let [r0, r1, r2] =
        Layout::vertical([Constraint::Length(2); 3]).areas(inner);

    let cpu = (stats.cpu_usage as f64).clamp(0.0, 100.0);
    let mem = pct(stats.memory_used, stats.memory_total);
    let swp = pct(stats.swap_used, stats.swap_total);

    render_gauge(frame, r0, "CPU ", cpu, None);
    render_gauge(frame, r1, "MEM ", mem, Some(format!(
        "{:.1}/{:.1} GB", to_gb(stats.memory_used), to_gb(stats.memory_total)
    )));
    render_gauge(frame, r2, "SWP ", swp, Some(format!(
        "{:.1}/{:.1} GB", to_gb(stats.swap_used), to_gb(stats.swap_total)
    )));
}

/// Unified gauge renderer
fn render_gauge(frame: &mut Frame, area: Rect, label: &str, pct: f64, detail: Option<String>) {
    let detail_width = if detail.is_some() { 22 } else { 8 };
    let [lbl_area, bar_area, txt_area] = Layout::horizontal([
        Constraint::Length(5),
        Constraint::Min(10),
        Constraint::Length(detail_width),
    ])
    .areas(area);

    frame.render_widget(
        Paragraph::new(Span::styled(label, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))),
        lbl_area,
    );
    frame.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(usage_color(pct)).bg(BAR_BG))
            .percent(pct as u16)
            .label(""),
        bar_area,
    );

    let txt = match detail {
        Some(d) => format!(" {pct:.1}%  {d}"),
        None => format!(" {pct:.1}%"),
    };
    frame.render_widget(
        Paragraph::new(Span::styled(txt, Style::default().fg(TEXT))),
        txt_area,
    );
}


fn draw_process_table(frame: &mut Frame, stats: &SystemStats, area: Rect) {
    let header = Row::new([hdr("PID"), hdr("CPU%"), hdr("MEM (MB)"), hdr("COMMAND")]);

    let rows = stats.processes.iter().enumerate().map(|(i, p)| {
        let bg = if i % 2 == 0 { Color::Reset } else { ROW_ALT };
        Row::new([
            Cell::from(format!("{}", p.pid)).style(Style::default().fg(TEXT)),
            Cell::from(format!("{:.1}", p.cpu_usage)).style(Style::default().fg(usage_color(p.cpu_usage as f64))),
            Cell::from(format!("{:.1}", p.memory_mb)).style(Style::default().fg(TEXT)),
            Cell::from(p.cmd.clone()).style(Style::default().fg(DIM)),
        ])
        .style(Style::default().bg(bg))
    });

    let table = Table::new(rows, [
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Min(20),
    ])
    .header(header)
    .block(styled_block("Top Processes"));

    frame.render_widget(table, area);
}


fn draw_bottom_row(frame: &mut Frame, stats: &SystemStats, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50); 2]).areas(area);

    // ── disks ──
    let disk_rows = stats.disk_usage.iter().map(|(name, used, total)| {
        Row::new([
            Cell::from(name.as_str()).style(Style::default().fg(TEXT)),
            Cell::from(format!("{:.1} GB", to_gb(*used))).style(Style::default().fg(TEXT)),
            Cell::from(format!("{:.1} GB", to_gb(*total))).style(Style::default().fg(DIM)),
        ])
    });
    frame.render_widget(
        Table::new(disk_rows, [Constraint::Min(12), Constraint::Length(10), Constraint::Length(10)])
            .header(Row::new([hdr("Name"), hdr("Used"), hdr("Total")]))
            .block(styled_block("Disks")),
        left,
    );

    // ── network ──
    let net_rows = stats.network.iter().map(|(iface, rx, tx)| {
        Row::new([
            Cell::from(iface.as_str()).style(Style::default().fg(TEXT)),
            Cell::from(SystemStats::format_bytes(*rx)).style(Style::default().fg(GOOD)),
            Cell::from(SystemStats::format_bytes(*tx)).style(Style::default().fg(WARN)),
        ])
    });
    frame.render_widget(
        Table::new(net_rows, [Constraint::Min(10), Constraint::Length(12), Constraint::Length(12)])
            .header(Row::new([hdr("Interface"), hdr("RX"), hdr("TX")]))
            .block(styled_block("Network")),
        right,
    );
}
