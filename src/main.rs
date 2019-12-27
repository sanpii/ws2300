use docopt::Docopt;

mod ws2300;

static USAGE: &str = "Usage: ws2300 <device>";

#[derive(serde_derive::Deserialize)]
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

    let args: Args = match docopt.deserialize() {
        Ok(args) => args,
        Err(e) => e.exit(),
    };

    let ws2300 = ws2300::Device::new(args.arg_device);

    let data = match ws2300.read_all() {
        Ok(data) => data,
        Err(err) => panic!("Read error: {}", err),
    };

    match serde_json::to_string(&data) {
        Ok(json) => println!("{}", json),
        Err(err) => panic!("JSON error: {}", err),
    };
}
