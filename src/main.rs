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

    match ws2300.read_all() {
        Ok(data) => println!("{:?}", data),
        Err(err) => panic!("Read error: {}", err),
    };
}
