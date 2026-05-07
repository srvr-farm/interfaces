use crate::interfaces::{discover_interfaces, display_rows, InterfaceInfo};
use crate::render::{self, MonitorRow};
use crate::stats::{calculate_rates, read_counters, Counters};
use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct MonitorSampler {
    sys_class_net: PathBuf,
    previous: HashMap<String, TimedCounters>,
}

#[derive(Debug, Clone, Copy)]
struct TimedCounters {
    counters: Counters,
    instant: Instant,
}

impl MonitorSampler {
    pub fn new(sys_class_net: impl AsRef<Path>) -> Self {
        Self {
            sys_class_net: sys_class_net.as_ref().to_path_buf(),
            previous: HashMap::new(),
        }
    }

    pub fn sample(&mut self, all: bool) -> anyhow::Result<Vec<MonitorRow>> {
        let interfaces = discover_interfaces()?;
        self.sample_from_interfaces(&interfaces, all, Instant::now())
            .context("sample interface counters")
    }

    pub fn sample_from_interfaces(
        &mut self,
        interfaces: &[InterfaceInfo],
        all: bool,
        now: Instant,
    ) -> io::Result<Vec<MonitorRow>> {
        let mut rates = HashMap::new();
        let mut active_interfaces = HashSet::new();

        for interface in interfaces {
            active_interfaces.insert(interface.name.clone());
            let current = read_counters(&self.sys_class_net, &interface.name)?;
            let Some(current) = current else {
                rates.insert(interface.name.clone(), (None, None));
                continue;
            };

            let rate = self.previous.get(&interface.name).and_then(|previous| {
                calculate_rates(previous.counters, current, now - previous.instant)
            });
            self.previous.insert(
                interface.name.clone(),
                TimedCounters {
                    counters: current,
                    instant: now,
                },
            );
            rates.insert(
                interface.name.clone(),
                rate.map(|rate| (Some(rate.rx_bytes_per_sec), Some(rate.tx_bytes_per_sec)))
                    .unwrap_or((None, None)),
            );
        }

        self.previous
            .retain(|interface_name, _| active_interfaces.contains(interface_name));

        Ok(display_rows(interfaces, all)
            .into_iter()
            .map(|row| {
                let (rx, tx) = rates
                    .get(&row.interface_name)
                    .copied()
                    .unwrap_or((None, None));
                MonitorRow::new(row, rx, tx)
            })
            .collect())
    }
}

pub fn run_monitor(all: bool, interval: Duration) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("enable raw mode")?;
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    let _guard = TerminalGuard;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("initialize terminal")?;
    terminal.clear().context("clear terminal")?;

    let mut sampler = MonitorSampler::new("/sys/class/net");
    let mut rows = sampler.sample(all)?;

    loop {
        terminal
            .draw(|frame| render::draw(frame, &rows, interval))
            .context("draw terminal frame")?;

        if event::poll(interval).context("poll terminal events")? {
            if let Event::Key(key) = event::read().context("read terminal event")? {
                if key.kind == KeyEventKind::Press && should_quit(key.code, key.modifiers) {
                    break;
                }
            }
        } else {
            rows = sampler.sample(all)?;
        }
    }

    Ok(())
}

pub fn should_quit(code: KeyCode, modifiers: KeyModifiers) -> bool {
    matches!(code, KeyCode::Esc)
        || matches!(code, KeyCode::Char('q'))
        || (matches!(code, KeyCode::Char('c')) && modifiers.contains(KeyModifiers::CONTROL))
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

#[cfg(test)]
mod tests {
    use super::{should_quit, MonitorSampler};
    use crate::interfaces::InterfaceInfo;
    use crossterm::event::{KeyCode, KeyModifiers};
    use std::fs;
    use std::net::Ipv4Addr;
    use std::time::{Duration, Instant};
    use tempfile::tempdir;

    #[test]
    fn should_quit_accepts_q_escape_and_ctrl_c() {
        assert!(should_quit(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(should_quit(KeyCode::Esc, KeyModifiers::NONE));
        assert!(should_quit(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!should_quit(KeyCode::Char('c'), KeyModifiers::NONE));
    }

    #[test]
    fn first_monitor_sample_has_no_rates() {
        let temp = tempdir().unwrap();
        write_counters(temp.path(), "eth0", 1_000, 2_000);
        let interfaces = vec![InterfaceInfo::new(
            "eth0",
            2,
            vec![Ipv4Addr::new(10, 0, 0, 10)],
        )];
        let mut sampler = MonitorSampler::new(temp.path());

        let rows = sampler
            .sample_from_interfaces(&interfaces, false, Instant::now())
            .unwrap();

        assert_eq!(rows[0].rx_bytes_per_sec, None);
        assert_eq!(rows[0].tx_bytes_per_sec, None);
    }

    #[test]
    fn second_monitor_sample_has_counter_rates() {
        let temp = tempdir().unwrap();
        write_counters(temp.path(), "eth0", 1_000, 2_000);
        let interfaces = vec![InterfaceInfo::new(
            "eth0",
            2,
            vec![Ipv4Addr::new(10, 0, 0, 10)],
        )];
        let mut sampler = MonitorSampler::new(temp.path());
        let start = Instant::now();
        sampler
            .sample_from_interfaces(&interfaces, false, start)
            .unwrap();
        write_counters(temp.path(), "eth0", 2_500, 5_000);

        let rows = sampler
            .sample_from_interfaces(&interfaces, false, start + Duration::from_millis(500))
            .unwrap();

        assert_eq!(rows[0].rx_bytes_per_sec, Some(3_000.0));
        assert_eq!(rows[0].tx_bytes_per_sec, Some(6_000.0));
    }

    fn write_counters(root: &std::path::Path, interface: &str, rx: u64, tx: u64) {
        let stats = root.join(interface).join("statistics");
        fs::create_dir_all(&stats).unwrap();
        fs::write(stats.join("rx_bytes"), format!("{rx}\n")).unwrap();
        fs::write(stats.join("tx_bytes"), format!("{tx}\n")).unwrap();
    }
}
