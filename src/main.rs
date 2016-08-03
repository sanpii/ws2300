extern crate serial;
extern crate docopt;
extern crate rustc_serialize;

use docopt::Docopt;

mod ws2300;

static USAGE: &'static str = "Usage: ws2300 <device>";

#[derive(RustcDecodable)]
struct Args
{
    arg_device: String,
}

fn main()
{
    let docopt = match Docopt::new(USAGE) {
        Ok(d) => d,
        Err(e) => e.exit(),
    };

    let args: Args = match docopt.decode() {
        Ok(args) => args,
        Err(e) => e.exit(),
    };

    let ws2300 = ws2300::Device::new(args.arg_device);

    match ws2300.temperature_indoor() {
        Ok(n) => println!("temperature_indoor: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.temperature_outdoor() {
        Ok(n) => println!("temperature_outdoor: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.dewpoint() {
        Ok(n) => println!("dewpoint: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.humidity_indoor() {
        Ok(n) => println!("humidity_indoor: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.humidity_outdoor() {
        Ok(n) => println!("humidity_outdoor: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.wind_speed() {
        Ok(n) => println!("wind_speed: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.wind_dir() {
        Ok(n) => println!("wind_dir: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.wind_direction() {
        Ok(n) => println!("wind_direction: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.wind_chill() {
        Ok(n) => println!("wind_chill: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.rain_1h() {
        Ok(n) => println!("rain_1h: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.rain_24h() {
        Ok(n) => println!("rain_24h: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.rain_total() {
        Ok(n) => println!("rain_total: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.pressure() {
        Ok(n) => println!("pressure: {}", n),
        Err(err) => panic!("Read error: {}", err),
    };
}
