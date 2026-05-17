use serde::Serialize;
use std::collections::HashSet;
use std::net::Ipv4Addr;
#[cfg(target_os = "windows")]
use std::process::Command;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct NetworkInterface {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub is_default: bool,
}

#[derive(Clone, Debug)]
pub struct NetworkState {
    interfaces: Vec<NetworkInterface>,
    selected_interface_id: Option<String>,
    manual_selection: bool,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
            selected_interface_id: None,
            manual_selection: false,
        }
    }
}

impl NetworkState {
    pub fn refresh(&mut self) {
        let interfaces = enumerate_network_interfaces();
        let preferred_id = preferred_interface_id(&interfaces);

        if self.manual_selection {
            let still_exists = self
                .selected_interface_id
                .as_ref()
                .is_some_and(|id| interfaces.iter().any(|item| &item.id == id));

            if !still_exists {
                self.manual_selection = false;
                self.selected_interface_id = preferred_id.clone();
            }
        } else {
            self.selected_interface_id = preferred_id.clone();
        }

        self.interfaces = interfaces;
    }

    pub fn select_interface(&mut self, interface_id: &str) -> bool {
        if self.interfaces.iter().any(|item| item.id == interface_id) {
            self.selected_interface_id = Some(interface_id.to_string());
            self.manual_selection = true;
            true
        } else {
            false
        }
    }

    pub fn interfaces(&self) -> Vec<NetworkInterface> {
        self.interfaces.clone()
    }

    pub fn selected_interface(&self) -> Option<NetworkInterface> {
        self.selected_interface_id.as_ref().and_then(|id| {
            self.interfaces
                .iter()
                .find(|item| &item.id == id)
                .cloned()
        })
    }

    pub fn selected_interface_id(&self) -> Option<String> {
        self.selected_interface_id.clone()
    }
}

fn enumerate_network_interfaces() -> Vec<NetworkInterface> {
    let default_ip = resolve_default_route_ip();
    let mut seen = HashSet::new();
    let mut interfaces = Vec::new();

    if let Ok(items) = local_ip_address::list_afinet_netifas() {
        for (name, ip) in items {
            let ipv4 = match ip {
                std::net::IpAddr::V4(value) if !value.is_loopback() => value,
                _ => continue,
            };

            let id = format!("{name}|{ipv4}");
            if !seen.insert(id.clone()) {
                continue;
            }

            interfaces.push(NetworkInterface {
                id,
                name,
                ip: ipv4.to_string(),
                is_default: default_ip.as_ref().is_some_and(|candidate| candidate == &ipv4),
            });
        }
    }

    interfaces.sort_by(|left, right| {
        right
            .is_default
            .cmp(&left.is_default)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.ip.cmp(&right.ip))
    });

    interfaces
}

fn preferred_interface_id(interfaces: &[NetworkInterface]) -> Option<String> {
    interfaces
        .iter()
        .find(|item| item.is_default)
        .or_else(|| interfaces.first())
        .map(|item| item.id.clone())
}

pub fn parse_windows_default_route_ip(route_output: &str) -> Option<Ipv4Addr> {
    let mut in_active_routes = false;

    for line in route_output.lines() {
        let trimmed = line.trim();

        if trimmed == "Active Routes:" {
            in_active_routes = true;
            continue;
        }

        if in_active_routes && trimmed.starts_with("Persistent Routes:") {
            break;
        }

        if !in_active_routes || trimmed.is_empty() || trimmed.starts_with("Network Destination") {
            continue;
        }

        let columns: Vec<&str> = trimmed.split_whitespace().collect();
        if columns.len() < 5 {
            continue;
        }

        if columns[0] == "0.0.0.0" && columns[1] == "0.0.0.0" {
            if let Ok(ip) = columns[3].parse::<Ipv4Addr>() {
                if !ip.is_loopback() {
                    return Some(ip);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn resolve_default_route_ip() -> Option<Ipv4Addr> {
    let output = Command::new("route")
        .args(["print", "0.0.0.0"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    parse_windows_default_route_ip(&stdout)
}

#[cfg(not(target_os = "windows"))]
fn resolve_default_route_ip() -> Option<Ipv4Addr> {
    None
}

#[cfg(test)]
mod tests {
    use super::{parse_windows_default_route_ip, NetworkInterface, NetworkState};
    use std::net::Ipv4Addr;

    #[test]
    fn parses_default_route_interface_ip_from_windows_route_output() {
        let output = r#"
===========================================================================
Interface List
===========================================================================

IPv4 Route Table
===========================================================================
Active Routes:
Network Destination        Netmask          Gateway       Interface  Metric
          0.0.0.0          0.0.0.0        10.31.0.1        10.31.0.6    200
        10.31.0.0      255.255.0.0         On-link         10.31.0.6    456
===========================================================================
Persistent Routes:
  None
"#;

        assert_eq!(
            parse_windows_default_route_ip(output),
            Some(Ipv4Addr::new(10, 31, 0, 6))
        );
    }

    #[test]
    fn returns_none_when_default_route_is_missing() {
        let output = r#"
IPv4 Route Table
===========================================================================
Active Routes:
Network Destination        Netmask          Gateway       Interface  Metric
      127.0.0.0        255.0.0.0         On-link         127.0.0.1    331
===========================================================================
"#;

        assert_eq!(parse_windows_default_route_ip(output), None);
    }

    #[test]
    fn keeps_manual_selection_when_interface_is_still_present() {
        let mut state = NetworkState::default();
        state.refresh();
        state.manual_selection = true;
        state.selected_interface_id = Some("wifi|10.31.0.6".to_string());
        state.interfaces = vec![
            NetworkInterface {
                id: "wifi|10.31.0.6".into(),
                name: "WLAN".into(),
                ip: "10.31.0.6".into(),
                is_default: false,
            },
            NetworkInterface {
                id: "hyperv|172.30.16.1".into(),
                name: "vEthernet".into(),
                ip: "172.30.16.1".into(),
                is_default: true,
            },
        ];

        assert!(state.select_interface("wifi|10.31.0.6"));
        assert_eq!(
            state.selected_interface().map(|item| item.ip),
            Some("10.31.0.6".to_string())
        );
    }
}
