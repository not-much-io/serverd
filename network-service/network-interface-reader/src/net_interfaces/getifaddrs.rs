use nix::{ifaddrs, sys::socket::SockAddr};

use crate::net_interfaces::{helpers, GetNetInterfaces, GetNetInterfacesResult, NetInterface};

use internal_prelude::library_prelude::*;

// https://man7.org/linux/man-pages/man3/getifaddrs.3.html
pub struct GetIfAddrs();

impl GetIfAddrs {
    pub fn new() -> GetIfAddrs {
        GetIfAddrs {}
    }
}

impl Default for GetIfAddrs {
    fn default() -> Self {
        GetIfAddrs::new()
    }
}

#[async_trait]
impl GetNetInterfaces for GetIfAddrs {
    async fn get_net_interfaces(&self) -> GetNetInterfacesResult {
        let mut net_interfaces = vec![];

        for iaddr in ifaddrs::getifaddrs()? {
            let name = iaddr.interface_name.clone();
            let addresses = match iaddr.address {
                Some(sock_addr) => match sock_addr {
                    SockAddr::Inet(inet_addr) => vec![inet_addr.ip().to_std()],
                    _ => {
                        log::info!(
                            "Unhandled Socket Address {} with flags: {:?}",
                            iaddr.interface_name,
                            iaddr.flags,
                        );
                        vec![]
                    }
                },
                None => vec![],
            };

            let ni = NetInterface::new(&name, addresses);
            net_interfaces.push(ni);
        }

        Ok(helpers::sort(helpers::normalize(net_interfaces)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn test_actual_call() {
        let mut names = HashSet::new();

        let nis = GetIfAddrs::default().get_net_interfaces().await.unwrap();
        assert!(!nis.is_empty(), "GetIfAddrs returned no network interfaces");
        for ni in nis {
            assert_ne!(ni.name, "", "network interface has no name");
            assert!(
                !names.contains(&ni.name),
                "duplicate network interface name: {}",
                ni.name
            );

            names.insert(ni.name);
        }
    }
}
