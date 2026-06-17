use facet::Facet;
use facet_yaml::{
    from_str
};
use std::fs;

#[derive(Facet)]
struct Preprocessing {
    block_size: i32,
    delta: i32,
}

#[derive(Facet)]
struct Config {
    qr_path: String,
    preprocessing: Preprocessing
}

fn main() {
    let config_file = fs::read_to_string("C:\\Users\\peica\\Desktop\\gamble_drv\\src\\bin\\config.yaml").expect("Open YAML config file");

    let cfg: Config = from_str(&config_file).unwrap();

    println!("{}", cfg.preprocessing.block_size);
}