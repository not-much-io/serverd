mod config_reader;

fn main() {
    let config = config_reader::read_config(None).expect("Unable to find serverd config");
    println!("{:?}", config)
}
