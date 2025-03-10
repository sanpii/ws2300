#![warn(warnings)]

use clap::Parser;

#[derive(Parser)]
struct Opt {
    device: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::parse();

    let ws2300 = ws2300::Device::new(opt.device)?;

    let data = match ws2300.read_all() {
        Ok(data) => data,
        Err(err) => panic!("Read error: {err}"),
    };

    match serde_json::to_string(&data) {
        Ok(json) => println!("{json}"),
        Err(err) => panic!("JSON error: {err}"),
    };

    Ok(())
}
