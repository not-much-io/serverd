pub mod getifaddrs;
pub mod ifconfig;
pub mod ip;

use std::fmt::Debug;
use std::net::IpAddr;

use internal_prelude::library_prelude::*;

#[async_trait]
pub trait GetNetInterfaces: Sync {
    async fn get_network_interfaces(&self) -> GetNetInterfacesResult;
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone)]
pub struct NetInterface {
    pub name:      String,
    pub addresses: Vec<IpAddr>,
}

impl NetInterface {
    fn new(name: &str, addresses: Vec<IpAddr>) -> Self {
        NetInterface {
            name: name.to_string(),
            addresses,
        }
    }
}

#[derive(Error, Debug)]
pub enum GetNetInterfacesError {
    #[error("No name found for network interface.")]
    NoNameForInterfaceFound(),
}

pub type GetNetInterfacesResult = Result<Vec<NetInterface>>;

mod helpers {
    use super::NetInterface;
    use std::collections::HashMap;

    pub fn normalize(net_interfaces: Vec<NetInterface>) -> Vec<NetInterface> {
        let mut name_to_ni: HashMap<String, NetInterface> = HashMap::new();

        for mut ni in net_interfaces {
            match name_to_ni.get_mut(&ni.name) {
                Some(entry) => {
                    entry.addresses.append(&mut ni.addresses);
                }
                None => {
                    name_to_ni.insert(ni.name.to_string(), ni);
                }
            }
        }

        name_to_ni.values().cloned().collect()
    }

    pub fn sort(mut net_interfaces: Vec<NetInterface>) -> Vec<NetInterface> {
        net_interfaces.sort_by(|a, b| Ord::cmp(&a.name, &b.name));
        net_interfaces
    }
}
