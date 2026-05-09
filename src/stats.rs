use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counters {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rates {
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateUnit {
    Bytes,
    Bits,
}

impl RateUnit {
    pub fn from_bits(bits: bool) -> Self {
        if bits {
            Self::Bits
        } else {
            Self::Bytes
        }
    }
}

pub fn read_counters(sys_class_net: &Path, interface: &str) -> io::Result<Option<Counters>> {
    let stats_dir = sys_class_net.join(interface).join("statistics");
    let rx_path = stats_dir.join("rx_bytes");
    let tx_path = stats_dir.join("tx_bytes");

    if !rx_path.exists() || !tx_path.exists() {
        return Ok(None);
    }

    let rx_bytes = read_counter(&rx_path)?;
    let tx_bytes = read_counter(&tx_path)?;

    Ok(Some(Counters { rx_bytes, tx_bytes }))
}

pub fn calculate_rates(previous: Counters, current: Counters, elapsed: Duration) -> Option<Rates> {
    if elapsed.is_zero()
        || current.rx_bytes < previous.rx_bytes
        || current.tx_bytes < previous.tx_bytes
    {
        return None;
    }

    let seconds = elapsed.as_secs_f64();
    Some(Rates {
        rx_bytes_per_sec: (current.rx_bytes - previous.rx_bytes) as f64 / seconds,
        tx_bytes_per_sec: (current.tx_bytes - previous.tx_bytes) as f64 / seconds,
    })
}

pub fn format_rate(rate: Option<f64>, unit: RateUnit) -> String {
    let Some(rate) = rate else {
        return "--".to_string();
    };

    match unit {
        RateUnit::Bytes => format_byte_rate(rate),
        RateUnit::Bits => format_bit_rate(rate * 8.0),
    }
}

fn format_byte_rate(rate: f64) -> String {
    if rate < 1024.0 {
        format!("{} B/s", rate.round() as u64)
    } else if rate < 1024.0 * 1024.0 {
        format!("{:.1} KiB/s", rate / 1024.0)
    } else if rate < 1024.0 * 1024.0 * 1024.0 {
        format!("{:.1} MiB/s", rate / 1024.0 / 1024.0)
    } else {
        format!("{:.1} GiB/s", rate / 1024.0 / 1024.0 / 1024.0)
    }
}

fn format_bit_rate(rate: f64) -> String {
    if rate < 1000.0 {
        format!("{} b/s", rate.round() as u64)
    } else if rate < 1000.0 * 1000.0 {
        format!("{:.1} Kb/s", rate / 1000.0)
    } else if rate < 1000.0 * 1000.0 * 1000.0 {
        format!("{:.1} Mb/s", rate / 1000.0 / 1000.0)
    } else {
        format!("{:.1} Gb/s", rate / 1000.0 / 1000.0 / 1000.0)
    }
}

fn read_counter(path: &Path) -> io::Result<u64> {
    let value = fs::read_to_string(path)?;
    value.trim().parse::<u64>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid counter {}: {error}", path.display()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{calculate_rates, format_rate, read_counters, Counters, RateUnit};
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn reads_rx_and_tx_counters_from_sysfs() {
        let temp = tempdir().unwrap();
        let stats = temp.path().join("eth0/statistics");
        fs::create_dir_all(&stats).unwrap();
        fs::write(stats.join("rx_bytes"), "1200\n").unwrap();
        fs::write(stats.join("tx_bytes"), "3400\n").unwrap();

        let counters = read_counters(temp.path(), "eth0").unwrap().unwrap();

        assert_eq!(
            counters,
            Counters {
                rx_bytes: 1200,
                tx_bytes: 3400
            }
        );
    }

    #[test]
    fn missing_counter_files_return_none() {
        let temp = tempdir().unwrap();

        assert_eq!(read_counters(temp.path(), "missing").unwrap(), None);
    }

    #[test]
    fn calculates_rates_from_counter_delta_and_elapsed_time() {
        let previous = Counters {
            rx_bytes: 1_000,
            tx_bytes: 5_000,
        };
        let current = Counters {
            rx_bytes: 2_500,
            tx_bytes: 9_000,
        };

        let rates = calculate_rates(previous, current, Duration::from_millis(500)).unwrap();

        assert_eq!(rates.rx_bytes_per_sec, 3_000.0);
        assert_eq!(rates.tx_bytes_per_sec, 8_000.0);
    }

    #[test]
    fn counter_reset_returns_none() {
        let previous = Counters {
            rx_bytes: 2_000,
            tx_bytes: 5_000,
        };
        let current = Counters {
            rx_bytes: 1_000,
            tx_bytes: 6_000,
        };

        assert_eq!(
            calculate_rates(previous, current, Duration::from_secs(1)),
            None
        );
    }

    #[test]
    fn formats_human_readable_rates() {
        assert_eq!(format_rate(None, RateUnit::Bytes), "--");
        assert_eq!(format_rate(Some(42.0), RateUnit::Bytes), "42 B/s");
        assert_eq!(format_rate(Some(1536.0), RateUnit::Bytes), "1.5 KiB/s");
        assert_eq!(
            format_rate(Some(2.0 * 1024.0 * 1024.0), RateUnit::Bytes),
            "2.0 MiB/s"
        );
    }

    #[test]
    fn formats_bit_rates_with_network_units() {
        assert_eq!(format_rate(None, RateUnit::Bits), "--");
        assert_eq!(format_rate(Some(42.0), RateUnit::Bits), "336 b/s");
        assert_eq!(format_rate(Some(125.0), RateUnit::Bits), "1.0 Kb/s");
        assert_eq!(format_rate(Some(1536.0), RateUnit::Bits), "12.3 Kb/s");
        assert_eq!(format_rate(Some(125_000.0), RateUnit::Bits), "1.0 Mb/s");
    }
}
