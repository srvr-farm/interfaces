use crate::interfaces::DisplayRow;
use crate::stats::{format_rate, RateUnit};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use std::fmt::Write;
use std::time::Duration;

const INTERFACE_WIDTH: usize = 16;
const IP_WIDTH: usize = 15;
const RATE_WIDTH: usize = 11;
const ORANGE: Color = Color::Indexed(208);

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

pub fn format_once_colored(rows: &[DisplayRow], no_headers: bool) -> String {
    let mut output = String::new();
    if !no_headers {
        writeln!(output, "{:<INTERFACE_WIDTH$} IP", "INTERFACE").unwrap();
    }

    for row in rows {
        writeln!(
            output,
            "{} {}",
            ansi_colored_name_column(row),
            row.ip_column
        )
        .unwrap();
    }

    output
}

pub fn format_monitor_text(rows: &[MonitorRow], rate_unit: RateUnit) -> String {
    let mut output = String::new();
    writeln!(
        output,
        "{:<INTERFACE_WIDTH$} {:<IP_WIDTH$} {:<RATE_WIDTH$} TX",
        "INTERFACE", "IP", "RX"
    )
    .unwrap();

    for row in rows {
        let (rx_rate, tx_rate) = primary_rates(row);
        writeln!(
            output,
            "{:<INTERFACE_WIDTH$} {:<IP_WIDTH$} {:<RATE_WIDTH$} {}",
            row.display.name_column,
            row.display.ip_column,
            format_rate(rx_rate, rate_unit),
            format_rate(tx_rate, rate_unit)
        )
        .unwrap();
    }

    output
}

pub fn monitor_text(rows: &[MonitorRow], rate_unit: RateUnit) -> Text<'static> {
    let mut lines = vec![Line::from(format!(
        "{:<INTERFACE_WIDTH$} {:<IP_WIDTH$} {:<RATE_WIDTH$} TX",
        "INTERFACE", "IP", "RX"
    ))];

    for row in rows {
        let (rx_rate, tx_rate) = primary_rates(row);
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<INTERFACE_WIDTH$}", row.display.name_column),
                Style::default().fg(interface_name_color(&row.display)),
            ),
            Span::raw(format!(
                " {:<IP_WIDTH$} {:<RATE_WIDTH$} {}",
                row.display.ip_column,
                format_rate(rx_rate, rate_unit),
                format_rate(tx_rate, rate_unit)
            )),
        ]));
    }

    Text::from(lines)
}

pub fn interface_name_color(row: &DisplayRow) -> Color {
    match (row.up, row.has_ip_address) {
        (true, true) => Color::Green,
        (true, false) => ORANGE,
        (false, _) => Color::Gray,
    }
}

fn ansi_colored_name_column(row: &DisplayRow) -> String {
    let name_column = format!("{:<INTERFACE_WIDTH$}", row.name_column);
    if row.name_column.is_empty() {
        return name_column;
    }

    format!(
        "\u{1b}[{}m{name_column}\u{1b}[0m",
        ansi_code(interface_name_color(row))
    )
}

fn ansi_code(color: Color) -> &'static str {
    match color {
        Color::Green => "32",
        Color::Gray => "37",
        ORANGE => "38;5;208",
        _ => "0",
    }
}

pub fn draw(frame: &mut Frame<'_>, rows: &[MonitorRow], interval: Duration, rate_unit: RateUnit) {
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
        Paragraph::new(monitor_text(rows, rate_unit))
            .block(Block::default().title("Interfaces").borders(Borders::ALL))
            .wrap(Wrap { trim: false }),
        root[1],
    );
}

fn primary_rates(row: &MonitorRow) -> (Option<f64>, Option<f64>) {
    if row.display.primary {
        (row.rx_bytes_per_sec, row.tx_bytes_per_sec)
    } else {
        (None, None)
    }
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
    use super::{
        format_monitor_text, format_once, format_once_colored, interface_name_color, monitor_text,
        MonitorRow,
    };
    use crate::interfaces::DisplayRow;
    use crate::stats::RateUnit;
    use ratatui::style::{Color, Style};

    fn row(interface_name: &str, name_column: &str, ip_column: &str, primary: bool) -> DisplayRow {
        row_with_state(
            interface_name,
            name_column,
            ip_column,
            primary,
            true,
            ip_column != "None",
        )
    }

    fn row_with_state(
        interface_name: &str,
        name_column: &str,
        ip_column: &str,
        primary: bool,
        up: bool,
        has_ip_address: bool,
    ) -> DisplayRow {
        DisplayRow {
            interface_name: interface_name.to_string(),
            name_column: name_column.to_string(),
            ip_column: ip_column.to_string(),
            primary,
            up,
            has_ip_address,
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
    fn formats_one_shot_output_with_colored_interface_names() {
        let rows = vec![
            row_with_state("eth0", "eth0", "10.0.0.10", true, true, true),
            row_with_state("eth1", "eth1", "None", true, false, false),
            row_with_state("eth2", "eth2", "None", true, true, false),
        ];

        assert_eq!(
            format_once_colored(&rows, false),
            concat!(
                "INTERFACE        IP\n",
                "\u{1b}[32meth0            \u{1b}[0m 10.0.0.10\n",
                "\u{1b}[37meth1            \u{1b}[0m None\n",
                "\u{1b}[38;5;208meth2            \u{1b}[0m None\n",
            )
        );
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
            format_monitor_text(&rows, RateUnit::Bytes),
            "INTERFACE        IP              RX          TX\neth0             10.0.0.10       1.5 KiB/s   42 B/s\n                 10.0.0.11       --          --\n"
        );
    }

    #[test]
    fn formats_monitor_output_as_network_bits_when_requested() {
        let rows = vec![MonitorRow::new(
            row("eth0", "eth0", "10.0.0.10", true),
            Some(1536.0),
            Some(125_000.0),
        )];

        assert_eq!(
            format_monitor_text(&rows, RateUnit::Bits),
            "INTERFACE        IP              RX          TX\neth0             10.0.0.10       12.3 Kb/s   1.0 Mb/s\n"
        );
    }

    #[test]
    fn chooses_interface_name_colors_from_state() {
        assert_eq!(
            interface_name_color(&row_with_state(
                "eth0",
                "eth0",
                "10.0.0.10",
                true,
                true,
                true
            )),
            Color::Green
        );
        assert_eq!(
            interface_name_color(&row_with_state("eth1", "eth1", "None", true, false, false)),
            Color::Gray
        );
        assert_eq!(
            interface_name_color(&row_with_state("eth2", "eth2", "None", true, true, false)),
            Color::Indexed(208)
        );
    }

    #[test]
    fn monitor_text_styles_interface_name_spans() {
        let rows = vec![
            MonitorRow::new(
                row_with_state("eth0", "eth0", "10.0.0.10", true, true, true),
                None,
                None,
            ),
            MonitorRow::new(
                row_with_state("eth1", "eth1", "None", true, false, false),
                None,
                None,
            ),
            MonitorRow::new(
                row_with_state("eth2", "eth2", "None", true, true, false),
                None,
                None,
            ),
        ];

        let text = monitor_text(&rows, RateUnit::Bytes);

        assert_eq!(
            text.lines[1].spans[0].style,
            Style::default().fg(Color::Green)
        );
        assert_eq!(
            text.lines[2].spans[0].style,
            Style::default().fg(Color::Gray)
        );
        assert_eq!(
            text.lines[3].spans[0].style,
            Style::default().fg(Color::Indexed(208))
        );
    }
}
