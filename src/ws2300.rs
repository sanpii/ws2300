use std;
use serial;
use std::cell::RefCell;
use std::thread::sleep;
use std::time::Duration;

pub struct Device
{
    port: Box<RefCell<serial::SerialPort>>,
}

impl Device
{
    pub fn new() -> Device
    {
        Device {
            port: Self::open(),
        }
    }

    fn open() -> Box<RefCell<serial::SerialPort>>
    {
        let device = "/dev/ttyUSB0";

        let mut port = match serial::open(device) {
            Ok(port) => port,
            Err(err) => panic!("Unable to open {}: {}.", device, err),
        };

        match Self::setup(&mut port) {
            Ok(_) => (),
            Err(err) => panic!("Setup error: {}", err),
        };

        Box::new(RefCell::new(port))
    }

    fn setup(port: &mut serial::SerialPort) -> serial::Result<()>
    {
        const SETTINGS: serial::PortSettings = serial::PortSettings {
            baud_rate: serial::Baud2400,
            char_size: serial::Bits8,
            flow_control: serial::FlowNone,
            parity: serial::ParityNone,
            stop_bits: serial::Stop1,
        };

        try!(
            port.configure(&SETTINGS)
        );
        try!(
            port.set_rts(true)
        );
        try!(
            port.set_dtr(false)
        );

        Ok(())
    }

    pub fn temperature_indoor(&self) -> serial::Result<f32>
    {
        let value = try!(
            self.try_read(0x346, 2)
        );

        let low = (value[0] >> 4) as f32 / 10.0 + (value[0] & 0xF) as f32 / 100.0;
        let high = (value[1] >> 4) as f32 * 10.0 + (value[1] & 0xF) as f32;

        Ok(Self::round(high + low - 30.0, 1))
    }

    fn try_read(&self, address: u32, size: usize) -> serial::Result<Vec<u8>>
    {
        for _ in 0..50 {
            match self.read(address, size) {
                Ok(n) => return Ok(n),
                Err(_) => (),
            };
        }

        Err(
            serial::Error::new(serial::ErrorKind::Io(std::io::ErrorKind::Other), "Try read error")
        )
    }

    fn read(&self, address: u32, size: usize) -> serial::Result<Vec<u8>>
    {
        let mut response: Vec<u8> = Vec::with_capacity(size);
        let mut buffer: [u8; 1] = [0; 1];
        let command = Self::encode_address(address, size);

        try!(
            self.reset()
        );

        for i in 0..5 {
            try!(
                self.port.borrow_mut().write(&[command[i]])
            );
            try!(
                self.port.borrow_mut().read_exact(&mut buffer[..])
            );
            try!(
                Self::check(command[i], i, buffer[0])
            );
        }

        for _ in 0..size {
            try!(
                self.port.borrow_mut().read_exact(&mut buffer[..])
            );

            response.push(buffer[0]);
        }

        try!(
            self.port.borrow_mut().read_exact(&mut buffer[..])
        );

        try!(
            Self::check_data(buffer[0], response.clone())
        );

        Ok(response)
    }

    fn check(command: u8, sequence: usize, answer: u8) -> serial::Result<()>
    {
        let checksum;

        if sequence < 4 {
            checksum = (sequence as u8) * 16 + (command - 0x82) / 4;
        }
        else {
            checksum = 0x30 + (command - 0xC2) / 4;
        }


        if checksum == answer {
            Ok(())
        }
        else {
            Err(
                serial::Error::new(serial::ErrorKind::Io(std::io::ErrorKind::Other), "Check error")
            )
        }
    }

    fn check_data(answer: u8, response: Vec<u8>) -> serial::Result<()>
    {
        let mut checksum: u8 = 0;

        for i in 0..response.len() {
            checksum += response[i];
        }

        checksum &= 0xFF;

        if checksum == answer {
            Ok(())
        }
        else {
            println!("{} {:?} {}", answer, response, checksum);
            Err(
                serial::Error::new(serial::ErrorKind::Io(std::io::ErrorKind::Other), "Check data error")
               )
        }
    }

    fn reset(&self) -> serial::Result<()>
    {
        let mut buffer: [u8; 1] = [0; 1];

        'outer: for _ in 0..100 {
            try!(
                self.port.borrow_mut().flush()
            );
            try!(
                self.port.borrow_mut().write(&[0x06])
            );

            sleep(Duration::from_millis(100));

            // FIXME possible infinite loop
            loop {
                match self.port.borrow_mut().read_exact(&mut buffer[..]) {
                    Ok(_) => {},
                    Err(_) => return Err(serial::Error::new(serial::ErrorKind::Io(std::io::ErrorKind::Other), "reset failed")),
                };

                if buffer[0] == 2 {
                    break 'outer;
                }
            }
        }

        Ok(())
    }

    fn encode_address(address: u32, number: usize) -> Vec<u8>
    {
        let mut command: Vec<u8> = vec![];

        if address == 0x06 {
            command = vec![0x06]
        }
        else {
            for i in 0..4 {
                let nibble = (address >> (4 * (3 - i))) & 0x0F;
                command.push(0x82 + (nibble * 4) as u8);
            }

            command.push(
                std::cmp::min(0xC2 + number * 4, 0xFE) as u8
            );
        }

        command
    }

    fn round(x: f32, n: u32) -> f32
    {
        let factor = 10u8.pow(n) as f32;
        let fract = (x.fract() * factor).round() / factor;

        x.trunc() + fract
    }
}

#[test]
fn test_address_encode()
{
    assert_eq!(Device::encode_address(0x06, 2), &[0x06]);
    assert_eq!(Device::encode_address(0x346, 2), &[130, 142, 146, 154, 202]);
}

#[test]
fn test_round()
{
    assert_eq!(Device::round(100.0, 2), 100.00);
    assert_eq!(Device::round(100.12345, 2), 100.12);
}
