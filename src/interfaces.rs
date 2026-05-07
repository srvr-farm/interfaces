use anyhow::Context;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use std::net::Ipv4Addr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceInfo {
    pub name: String,
    pub index: u32,
    pub ipv4_addresses: Vec<Ipv4Addr>,
}

impl InterfaceInfo {
    pub fn new(name: impl Into<String>, index: u32, ipv4_addresses: Vec<Ipv4Addr>) -> Self {
        Self {
            name: name.into(),
            index,
            ipv4_addresses,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayRow {
    pub interface_name: String,
    pub name_column: String,
    pub ip_column: String,
    pub primary: bool,
}

pub fn discover_interfaces() -> anyhow::Result<Vec<InterfaceInfo>> {
    let interfaces = NetworkInterface::show().context("list network interfaces")?;
    Ok(interfaces
        .into_iter()
        .map(interface_info_from_network_interface)
        .collect())
}

pub fn display_rows(interfaces: &[InterfaceInfo], all: bool) -> Vec<DisplayRow> {
    let mut interfaces = interfaces.to_vec();
    interfaces.sort_by(|left, right| {
        left.index
            .cmp(&right.index)
            .then_with(|| left.name.cmp(&right.name))
    });

    interfaces
        .iter()
        .flat_map(|interface| rows_for_interface(interface, all))
        .collect()
}

fn interface_info_from_network_interface(interface: NetworkInterface) -> InterfaceInfo {
    let ipv4_addresses = interface
        .addr
        .into_iter()
        .filter_map(|addr| match addr {
            Addr::V4(v4) => Some(v4.ip),
            Addr::V6(_) => None,
        })
        .collect();

    InterfaceInfo::new(interface.name, interface.index, ipv4_addresses)
}

fn rows_for_interface(interface: &InterfaceInfo, all: bool) -> Vec<DisplayRow> {
    if interface.ipv4_addresses.is_empty() {
        return if all {
            vec![DisplayRow {
                interface_name: interface.name.clone(),
                name_column: interface.name.clone(),
                ip_column: "None".to_string(),
                primary: true,
            }]
        } else {
            Vec::new()
        };
    }

    interface
        .ipv4_addresses
        .iter()
        .enumerate()
        .map(|(index, ip)| DisplayRow {
            interface_name: interface.name.clone(),
            name_column: if index == 0 {
                interface.name.clone()
            } else {
                String::new()
            },
            ip_column: ip.to_string(),
            primary: index == 0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{display_rows, InterfaceInfo};
    use std::net::Ipv4Addr;

    #[test]
    fn display_rows_skip_interfaces_without_ipv4_by_default() {
        let interfaces = vec![
            InterfaceInfo::new("lo", 1, vec![Ipv4Addr::new(127, 0, 0, 1)]),
            InterfaceInfo::new("eth0", 2, vec![Ipv4Addr::new(10, 0, 0, 10)]),
            InterfaceInfo::new("tun0", 3, Vec::new()),
        ];

        let rows = display_rows(&interfaces, false);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name_column, "lo");
        assert_eq!(rows[0].ip_column, "127.0.0.1");
        assert_eq!(rows[1].name_column, "eth0");
        assert_eq!(rows[1].ip_column, "10.0.0.10");
    }

    #[test]
    fn display_rows_include_interfaces_without_ipv4_when_all_is_set() {
        let interfaces = vec![InterfaceInfo::new("tun0", 3, Vec::new())];

        let rows = display_rows(&interfaces, true);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].interface_name, "tun0");
        assert_eq!(rows[0].name_column, "tun0");
        assert_eq!(rows[0].ip_column, "None");
    }

    #[test]
    fn display_rows_blank_repeated_interface_names_for_extra_addresses() {
        let interfaces = vec![InterfaceInfo::new(
            "eth0",
            2,
            vec![Ipv4Addr::new(10, 0, 0, 10), Ipv4Addr::new(10, 0, 0, 11)],
        )];

        let rows = display_rows(&interfaces, false);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name_column, "eth0");
        assert_eq!(rows[0].ip_column, "10.0.0.10");
        assert_eq!(rows[1].interface_name, "eth0");
        assert_eq!(rows[1].name_column, "");
        assert_eq!(rows[1].ip_column, "10.0.0.11");
    }

    #[test]
    fn display_rows_are_sorted_by_index_then_name() {
        let interfaces = vec![
            InterfaceInfo::new("eth1", 3, vec![Ipv4Addr::new(10, 0, 0, 2)]),
            InterfaceInfo::new("lo", 1, vec![Ipv4Addr::new(127, 0, 0, 1)]),
            InterfaceInfo::new("eth0", 2, vec![Ipv4Addr::new(10, 0, 0, 1)]),
        ];

        let rows = display_rows(&interfaces, false);

        assert_eq!(
            rows.iter()
                .map(|row| row.interface_name.as_str())
                .collect::<Vec<_>>(),
            vec!["lo", "eth0", "eth1"]
        );
    }
}
