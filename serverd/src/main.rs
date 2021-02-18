mod config_reader;
// use monitoring_service::PollingMonitor;
// use networking_service::{network_interfaces, public_ip};

fn main() {
    let _ = config_reader::read_config(None).expect("Unable to find serverd config");

    // PollingMonitor::new(cmd_to_monitor, cmd_to_trigger)
}
