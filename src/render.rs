use crate::interfaces::DisplayRow;
use crate::stats::format_rate;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::fmt::Write;
use std::time::Duration;

const INTERFACE_WIDTH: usize = 16;
const IP_WIDTH: usize = 15;
const RATE_WIDTH: usize = 11;

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorRow {
    pub display: DisplayRow,
    pub rx_bytes_per_sec: Option<f64>,
    pub tx_bytes_per_sec: Option<f64>,
}

impl MonitorRow {
    pub fn new(
        display: DisplayRow,
        rx_bytes_per_sec: Option<f64>,
        tx_bytes_per_sec: Option<f64>,
    ) -> Self {
        Self {
            display,
            rx_bytes_per_sec,
            tx_bytes_per_sec,
        }
    }
}

pub fn format_once(rows: &[DisplayRow], no_headers: bool) -> String {
    let mut output = String::new();
    if !no_headers {
        writeln!(output, "{:<INTERFACE_WIDTH$} IP", "INTERFACE").unwrap();
    }

    for row in rows {
        writeln!(
            output,
            "{:<INTERFACE_WIDTH$} {}",
            row.name_column, row.ip_column
        )
        .unwrap();
    }

    output
}

pub fn format_monitor_text(rows: &[MonitorRow]) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "{:<INTERFACE_WIDTH$} {:<IP_WIDTH$} {:<RATE_WIDTH$} TX",
        "INTERFACE", "IP", "RX"
    )
    .unwrap();

    for row in rows {
        let (rx_rate, tx_rate) = if row.display.primary {
            (row.rx_bytes_per_sec, row.tx_bytes_per_sec)
        } else {
            (None, None)
        };
        writeln!(
            output,
            "{:<INTERFACE_WIDTH$} {:<IP_WIDTH$} {:<RATE_WIDTH$} {}",
            row.display.name_column,
            row.display.ip_column,
            format_rate(rx_rate),
            format_rate(tx_rate)
        )
        .unwrap();
    }

    output
}

pub fn draw(frame: &mut Frame<'_>, rows: &[MonitorRow], interval: Duration) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(frame.area());

    let title = Paragraph::new(format!(
        "ifs  interval={}s  q/Esc/Ctrl-C to quit",
        trim_float(interval.as_secs_f64())
    ))
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, root[0]);

    frame.render_widget(
        Paragraph::new(format_monitor_text(rows))
            .block(Block::default().title("Interfaces").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        root[1],
    );
}

fn trim_float(value: f64) -> String {
    let formatted = format!("{value:.3}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{format_monitor_text, format_once, MonitorRow};
    use crate::interfaces::DisplayRow;

    fn row(interface_name: &str, name_column: &str, ip_column: &str, primary: bool) -> DisplayRow {
        DisplayRow {
            interface_name: interface_name.to_string(),
            name_column: name_column.to_string(),
            ip_column: ip_column.to_string(),
            primary,
        }
    }

    #[test]
    fn formats_one_shot_output_with_headers() {
        let rows = vec![
            row("lo", "lo", "127.0.0.1", true),
            row("eth0", "eth0", "10.0.0.10", true),
        ];

        assert_eq!(
            format_once(&rows, false),
            "INTERFACE        IP\nlo               127.0.0.1\neth0             10.0.0.10\n"
        );
    }

    #[test]
    fn formats_one_shot_output_without_headers() {
        let rows = vec![row("lo", "lo", "127.0.0.1", true)];

        assert_eq!(format_once(&rows, true), "lo               127.0.0.1\n");
    }

    #[test]
    fn formats_monitor_output_with_rates_only_on_primary_rows() {
        let rows = vec![
            MonitorRow::new(
                row("eth0", "eth0", "10.0.0.10", true),
                Some(1536.0),
                Some(42.0),
            ),
            MonitorRow::new(
                row("eth0", "", "10.0.0.11", false),
                Some(1536.0),
                Some(42.0),
            ),
        ];

        assert_eq!(
            format_monitor_text(&rows),
            "INTERFACE        IP              RX          TX\neth0             10.0.0.10       1.5 KiB/s   42 B/s\n                 10.0.0.11       --          --\n"
        );
    }
}
