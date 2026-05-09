use clap::{ArgAction, Parser};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Once,
    Monitor(Duration),
}

#[derive(Debug, Clone, Parser)]
#[command(name = "ifs", disable_help_flag = true)]
pub struct Cli {
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub bits: bool,
    #[arg(short = 'h', long)]
    pub no_headers: bool,
    #[arg(short = 'i', long = "interval", value_name = "SECONDS", num_args = 0..=1, default_missing_value = "1")]
    interval: Option<String>,
    #[arg(long = "help", action = ArgAction::Help)]
    help: Option<bool>,
}

impl Cli {
    pub fn mode(&self) -> Result<Mode, String> {
        self.interval
            .as_deref()
            .map(parse_interval)
            .transpose()
            .map(|interval| interval.map_or(Mode::Once, Mode::Monitor))
    }
}

fn parse_interval(value: &str) -> Result<Duration, String> {
    let value = value.trim();
    let duration = if let Some(milliseconds) = value.strip_suffix("ms") {
        let milliseconds = milliseconds
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("invalid interval: {value}"))?;
        Duration::from_secs_f64(milliseconds / 1000.0)
    } else if let Some(seconds) = value.strip_suffix('s') {
        let seconds = seconds
            .trim()
            .parse::<f64>()
            .map_err(|_| format!("invalid interval: {value}"))?;
        Duration::from_secs_f64(seconds)
    } else {
        let seconds = value
            .parse::<f64>()
            .map_err(|_| format!("invalid interval: {value}"))?;
        Duration::from_secs_f64(seconds)
    };

    if duration.is_zero() {
        return Err("interval must be greater than zero".to_string());
    }

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::{Cli, Mode};
    use clap::Parser;
    use std::time::Duration;

    #[test]
    fn defaults_to_instant_one_shot_without_all_interfaces() {
        let cli = Cli::try_parse_from(["ifs"]).unwrap();

        assert_eq!(cli.mode().unwrap(), Mode::Once);
        assert!(!cli.all);
        assert!(!cli.bits);
        assert!(!cli.no_headers);
    }

    #[test]
    fn parses_existing_all_and_no_header_flags() {
        let cli = Cli::try_parse_from(["ifs", "--all", "--no-headers"]).unwrap();

        assert!(cli.all);
        assert!(cli.no_headers);
    }

    #[test]
    fn parses_bits_flag() {
        let cli = Cli::try_parse_from(["ifs", "--bits"]).unwrap();

        assert!(cli.bits);
    }

    #[test]
    fn short_h_keeps_existing_no_header_behavior() {
        let cli = Cli::try_parse_from(["ifs", "-h"]).unwrap();

        assert!(cli.no_headers);
    }

    #[test]
    fn bare_short_interval_uses_default_one_second_refresh() {
        let cli = Cli::try_parse_from(["ifs", "-i"]).unwrap();

        assert_eq!(cli.mode().unwrap(), Mode::Monitor(Duration::from_secs(1)));
    }

    #[test]
    fn short_interval_accepts_fractional_seconds() {
        let cli = Cli::try_parse_from(["ifs", "-i", "0.5"]).unwrap();

        assert_eq!(
            cli.mode().unwrap(),
            Mode::Monitor(Duration::from_millis(500))
        );
    }

    #[test]
    fn long_interval_accepts_seconds() {
        let cli = Cli::try_parse_from(["ifs", "--interval", "3"]).unwrap();

        assert_eq!(cli.mode().unwrap(), Mode::Monitor(Duration::from_secs(3)));
    }

    #[test]
    fn rejects_zero_interval() {
        let cli = Cli::try_parse_from(["ifs", "-i", "0"]).unwrap();
        let error = cli.mode().unwrap_err();

        assert!(error.contains("interval must be greater than zero"));
    }
}
