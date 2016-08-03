extern crate serial;

mod ws2300;

fn main()
{
    let ws2300 = ws2300::Device::new();

    match ws2300.temperature_indoor() {
        Ok(n) => println!("{:?}", n),
        Err(err) => panic!("Read error: {}", err),
    };

    match ws2300.temperature_outdoor() {
        Ok(n) => println!("{:?}", n),
        Err(err) => panic!("Read error: {}", err),
    };
}
