use std::net::IpAddr;
use std::process::{Command, Output};

use nursery_prelude::library_prelude::*;

use crate::net_interfaces::{
    GetNetInterfaces, GetNetInterfacesError, GetNetInterfacesResult, NetInterface,
};

// https://man7.org/linux/man-pages/man8/ifconfig.8.html
pub struct IfConfig();

impl IfConfig {
    fn new() -> Self {
        IfConfig {}
    }
}

impl Default for IfConfig {
    fn default() -> Self {
        IfConfig::new()
    }
}

#[async_trait]
impl GetNetInterfaces for IfConfig {
    async fn get_net_interfaces(&self) -> GetNetInterfacesResult {
        self.parse_output(self.call().await?).await
    }
}

#[async_trait]
impl CLIProgram<GetNetInterfacesResult> for IfConfig {
    fn name(&self) -> &str {
        "ifconfig"
    }

    async fn call(&self) -> Result<Output> {
        Ok(Command::new(self.name()).arg("-a").output()?)
    }

    async fn parse_output(&self, output: Output) -> GetNetInterfacesResult {
        let mut net_interfaces = vec![];

        let stdout = String::from_utf8(output.stdout)?;
        for c in RE.captures_iter(&stdout) {
            let interface_name = get_interface_name(&c)?;
            net_interfaces.push(NetInterface::new(
                &interface_name.to_string(),
                get_ip_addresses(&c)?,
            ));
        }

        Ok(net_interfaces)
    }
}

fn get_interface_name(c: &regex::Captures) -> Result<String> {
    Ok(c.name(REGEX_GROUP_NAME)
        .ok_or_else(GetNetInterfacesError::NoNameForInterfaceFound)?
        .as_str()
        .into())
}

fn get_ip_addresses(c: &regex::Captures) -> Result<Vec<IpAddr>> {
    let parse_ip_addr = |m: regex::Match| m.as_str().parse::<IpAddr>().ok();

    let mut res: Vec<IpAddr> = vec![];

    if let Some(m) = c.name(REGEX_GROUP_IPV4) {
        if let Some(addr) = parse_ip_addr(m) {
            res.push(addr);
        }
    }

    if let Some(m) = c.name(REGEX_GROUP_IPV6) {
        if let Some(addr) = parse_ip_addr(m) {
            res.push(addr);
        }
    }

    Ok(res)
}

const REGEX_GROUP_NAME: &str = "interface_name";
const REGEX_GROUP_IPV4: &str = "interface_ip_v4";
const REGEX_GROUP_IPV6: &str = "interface_ip_v6";

lazy_static! {
    /// Regex to get all the data from the ifconfig command output
    /// TODO: Pretty hard to grok, some way to simplify, explain, format?
    static ref RE: regex::Regex = regex::Regex::new(r#"(?P<interface_name>.*?): (?:[\S\s]*?inet (?P<interface_ip_v4>.*?)  netmask){0,1}(?:[\S\s]*?(?:RX|inet6 (?P<interface_ip_v6>.*?)  prefixlen)){0,1}"#).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    const IFCONFIG_OUTPUT: &str = "
br-b83013461f0c: flags=4099<UP,BROADCAST,MULTICAST>  mtu 1500
    inet 172.23.0.1  netmask 255.255.0.0  broadcast 172.23.255.255
    ether 02:42:5d:8c:83:bc  txqueuelen 0  (Ethernet)
    RX packets 0  bytes 0 (0.0 B)
    RX errors 0  dropped 0  overruns 0  frame 0
    TX packets 0  bytes 0 (0.0 B)
    TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

docker0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
    inet 172.17.0.1  netmask 255.255.0.0  broadcast 172.17.255.255
    inet6 fe80::42:79ff:fe2b:f5c3  prefixlen 64  scopeid 0x20<link>
    ether 02:42:79:2b:f5:c3  txqueuelen 0  (Ethernet)
    RX packets 199772  bytes 10417009 (9.9 MiB)
    RX errors 0  dropped 0  overruns 0  frame 0
    TX packets 376100  bytes 558495778 (532.6 MiB)
    TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

enp34s0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
    inet 192.168.0.11  netmask 255.255.255.0  broadcast 192.168.0.255
    inet6 fe80::6954:9b0a:f51f:e14e  prefixlen 64  scopeid 0x20<link>
    ether 00:d8:61:a9:da:ea  txqueuelen 1000  (Ethernet)
    RX packets 1910190  bytes 2676122114 (2.4 GiB)
    RX errors 0  dropped 0  overruns 0  frame 0
    TX packets 1091610  bytes 93109462 (88.7 MiB)
    TX errors 8  dropped 0 overruns 0  carrier 4  collisions 50576

lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536
    inet 127.0.0.1  netmask 255.0.0.0
    inet6 ::1  prefixlen 128  scopeid 0x10<host>
    loop  txqueuelen 1000  (Local Loopback)
    RX packets 426238  bytes 327391145 (312.2 MiB)
    RX errors 0  dropped 0  overruns 0  frame 0
    TX packets 426238  bytes 327391145 (312.2 MiB)
    TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

veth60de6b9: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
    inet6 fe80::d833:3eff:fe68:3a08  prefixlen 64  scopeid 0x20<link>
    ether da:33:3e:68:3a:08  txqueuelen 0  (Ethernet)
    RX packets 38566  bytes 2547730 (2.4 MiB)
    RX errors 0  dropped 0  overruns 0  frame 0
    TX packets 72584  bytes 107243713 (102.2 MiB)
    TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

";

    #[tokio::test]
    async fn parse_output() {
        let output = Output {
            status: ExitStatus::from_raw(0),
            stderr: Vec::new(),
            stdout: IFCONFIG_OUTPUT.into(),
        };
        let real = IfConfig().parse_output(output).await.unwrap();
        let expected = vec![
            (
                "br-b83013461f0c",
                vec!["172.23.0.1".parse::<IpAddr>().unwrap()],
            ),
            (
                "docker0",
                vec![
                    "172.17.0.1".parse::<IpAddr>().unwrap(),
                    "fe80::42:79ff:fe2b:f5c3".parse::<IpAddr>().unwrap(),
                ],
            ),
            (
                "enp34s0",
                vec![
                    "192.168.0.11".parse::<IpAddr>().unwrap(),
                    "fe80::6954:9b0a:f51f:e14e".parse::<IpAddr>().unwrap(),
                ],
            ),
            (
                "lo",
                vec![
                    "127.0.0.1".parse::<IpAddr>().unwrap(),
                    "::1".parse::<IpAddr>().unwrap(),
                ],
            ),
            (
                "veth60de6b9",
                vec!["fe80::d833:3eff:fe68:3a08".parse::<IpAddr>().unwrap()],
            ),
        ];

        for (i, (name, addresses)) in expected.iter().enumerate() {
            let net_interface = real.get(i).unwrap();

            assert_eq!(*name, net_interface.name);
            assert_eq!(*addresses, net_interface.addresses);
        }
    }

    #[tokio::test]
    async fn test_actual_call() {
        let ifconfig = IfConfig();

        assert!(
            ifconfig.is_installed(),
            "ifconfig not installed in environment"
        );

        let interfaces = ifconfig.get_net_interfaces().await.unwrap();

        assert!(
            !interfaces.is_empty(),
            "No network interfaces at all in this environment. Required for test."
        );
        for interface in interfaces {
            assert_ne!(interface.name, "", "Network interface name empty.");
        }
    }
}
