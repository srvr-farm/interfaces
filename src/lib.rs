pub mod cli;
pub mod interfaces;
pub mod render;
pub mod stats;
pub mod tui;

use crate::cli::{Cli, Mode};
use crate::interfaces::{discover_interfaces, display_rows, InterfaceInfo};
use anyhow::anyhow;
use clap::Parser;

pub fn run() -> anyhow::Result<()> {
    run_with_cli(Cli::parse())
}

pub fn run_with_cli(cli: Cli) -> anyhow::Result<()> {
    match cli.mode().map_err(|error| anyhow!(error))? {
        Mode::Once => {
            print!("{}", one_shot_output(cli.all, cli.no_headers)?);
            Ok(())
        }
        Mode::Monitor(interval) => tui::run_monitor(cli.all, interval),
    }
}

fn one_shot_output(all: bool, no_headers: bool) -> anyhow::Result<String> {
    let interfaces = discover_interfaces()?;
    Ok(one_shot_output_from_interfaces(
        &interfaces,
        all,
        no_headers,
    ))
}

fn one_shot_output_from_interfaces(
    interfaces: &[InterfaceInfo],
    all: bool,
    no_headers: bool,
) -> String {
    let rows = display_rows(interfaces, all);
    render::format_once(&rows, no_headers)
}

#[cfg(test)]
mod tests {
    use super::one_shot_output_from_interfaces;
    use crate::interfaces::InterfaceInfo;
    use std::net::Ipv4Addr;

    #[test]
    fn one_shot_output_keeps_existing_two_column_shape() {
        let interfaces = vec![InterfaceInfo::new(
            "eth0",
            2,
            vec![Ipv4Addr::new(10, 0, 0, 10)],
        )];

        let output = one_shot_output_from_interfaces(&interfaces, false, false);

        assert_eq!(output, "INTERFACE        IP\neth0             10.0.0.10\n");
        assert!(!output.contains("RX"));
        assert!(!output.contains("TX"));
    }
}
