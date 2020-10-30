use std::net::IpAddr;

use nursery_prelude::application_prelude::*;

use nety::net_interfaces::{getifaddrs::GetIfAddrs, ifconfig::IfConfig, ip::Ip};
use nety::net_interfaces::{GetNetInterfaces, GetNetInterfacesResult, NetInterface};
use nety::public_ip::dig::Dig;
use nety::public_ip::{GetPublicIP, GetPublicIPResult};

#[tokio::main]
async fn main() {
    let key_verbose_short = 'v';
    let key_verbose = "verbose";
    let matches = clap::App::new("nety")
        .version("0.1")
        .author("kristo.koert@gmail.com")
        .about("A tool for gathering networking related information")
        .arg(
            clap::Arg::new(key_verbose)
                .short(key_verbose_short)
                .long(key_verbose)
                .about("Run in verbose more showing various debugging output")
                .required(false)
                .takes_value(false),
        )
        .get_matches();

    if matches.is_present(key_verbose) {
        env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .init();
    } else {
        env_logger::builder()
            .filter_level(log::LevelFilter::Error)
            .init();
    }

    let (public_ip_res, network_interfaces_res) =
        tokio::join!(get_public_ip(), get_net_interfaces());

    match public_ip_res {
        Ok(public_ip) => display_public_ip(public_ip),
        Err(err) => log::error!("{}", err),
    }

    match network_interfaces_res {
        Ok(net_interfaces) => display_network_interfaces(net_interfaces),
        Err(err) => log::error!("{}", err),
    }
}

#[derive(Error, Debug)]
pub enum NetyError {
    #[error("No tools for getting network interfaces installed")]
    NoGetNetInterfacesMethodSucceeded,
    #[error("No tools for getting the public ip installed")]
    NoGetPublicIPToolInstalled,
}

lazy_static! {
    // Public ip tools in priority order
    static ref GET_PUBLIC_IP_TOOLS: [Box<dyn GetPublicIP>; 1] = [
        Box::new(Dig::default()),
    ];

    // Network interface tools in priority order
    static ref GET_NET_INTERFACE_TOOLS: [Box<dyn GetNetInterfaces>; 3] =
        [
            Box::new(GetIfAddrs::default()), // libc based implementation
            Box::new(Ip::default()),         // Defacto linux networking utility
            Box::new(IfConfig::default()),   // Defacto linux networking utility (deprecated)
        ];
}

async fn get_public_ip() -> GetPublicIPResult {
    if let Some(t) = GET_PUBLIC_IP_TOOLS.iter().find(|t| t.is_installed()) {
        return t.get_public_ip().await;
    }

    Err(NetyError::NoGetPublicIPToolInstalled.into())
}

async fn get_net_interfaces() -> GetNetInterfacesResult {
    for method in GET_NET_INTERFACE_TOOLS.iter() {
        let res = method.get_net_interfaces().await;
        if res.is_ok() {
            return res;
        }
    }

    Err(NetyError::NoGetNetInterfacesMethodSucceeded.into())
}

fn display_public_ip(ip: IpAddr) {
    println!("Public IP: {}", ip)
}

fn display_network_interfaces(net_interfaces: Vec<NetInterface>) {
    println!("Network Interfaces:");
    for ni in net_interfaces {
        println!("  Name: {}", ni.name);
        println!("  Addresses:");
        for address in ni.addresses {
            println!("    {}", address.to_string());
        }
        println!();
    }
}
