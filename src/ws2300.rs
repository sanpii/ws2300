use std;
use serial;
use std::cell::RefCell;
use std::thread::sleep;
use std::time::Duration;

pub struct Device
{
    port: Box<RefCell<serial::SerialPort>>,
    memory: MemoryMap,
}

struct MemoryMap
{
    temperature_indoor: Memory,
    temperature_outdoor: Memory,
    dewpoint: Memory,
    humidity_indoor: Memory,
    humidity_outdoor: Memory,
    wind_speed: Memory,
    wind_dir: Memory,
    wind_chill: Memory,
    rain_1h: Memory,
    rain_24h: Memory,
    rain_total: Memory,
    pressure: Memory,
}

struct Memory
{
    address: u32,
    size: usize
}

impl Device
{
    pub fn new(device: String) -> Device
    {
        let memory = MemoryMap {
            temperature_indoor: Memory {address: 0x346, size: 2},
            temperature_outdoor: Memory {address: 0x373, size: 2},
            dewpoint: Memory {address: 0x3CE, size: 2},
            humidity_indoor: Memory {address: 0x3FB, size: 1},
            humidity_outdoor: Memory {address: 0x419, size: 1},
            wind_speed: Memory {address: 0x529, size: 3},
            wind_dir: Memory {address: 0x52C, size: 1},
            wind_chill: Memory {address: 0x3A0, size: 2},
            rain_1h: Memory {address: 0x4B4, size: 3},
            rain_24h: Memory {address: 0x497, size: 3},
            rain_total: Memory {address: 0x4D2, size: 3},
            pressure: Memory {address: 0x5E2, size: 3},
        };

        Device {
            port: Self::open(device),
            memory: memory,
        }
    }

    fn open(device: String) -> Box<RefCell<serial::SerialPort>>
    {
        let mut port = match serial::open(&device) {
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
        self.temperature(&self.memory.temperature_indoor)
    }

    pub fn temperature_outdoor(&self) -> serial::Result<f32>
    {
        self.temperature(&self.memory.temperature_outdoor)
    }

    pub fn dewpoint(&self) -> serial::Result<f32>
    {
        self.temperature(&self.memory.dewpoint)
    }

    fn temperature(&self, memory: &Memory) -> serial::Result<f32>
    {
        let value = try!(
            self.try_read(memory)
        );

        let low = (value[0] >> 4) as f32 / 10.0 + (value[0] & 0xF) as f32 / 100.0;
        let high = (value[1] >> 4) as f32 * 10.0 + (value[1] & 0xF) as f32;

        Ok(Self::round(high + low - 30.0, 1))
    }

    pub fn humidity_indoor(&self) -> serial::Result<u32>
    {
        self.humidity(&self.memory.humidity_indoor)
    }

    pub fn humidity_outdoor(&self) -> serial::Result<u32>
    {
        self.humidity(&self.memory.humidity_outdoor)
    }

    fn humidity(&self, memory: &Memory) -> serial::Result<u32>
    {
        let value = try!(
            self.try_read(memory)
        );

        let low = (value[0] >> 4) as u32 * 10 + (value[0] & 0xF) as u32;

        Ok(low)
    }

    pub fn wind_speed(&self) -> serial::Result<f32>
    {
        let value = try!(
            self.try_read(&self.memory.wind_speed)
        );

        Ok(((((value[1] & 0xF) as u16) << 8) as f32 + value[0] as f32) / 10.0)
    }

    pub fn wind_dir(&self) -> serial::Result<f32>
    {
        let value = try!(
            self.try_read(&self.memory.wind_dir)
        );

        let low = (value[0] >> 4) as f32;

        Ok(Self::round(low * 22.5, 1))
    }

    pub fn wind_direction(&self) -> serial::Result<String>
    {
        let directions: Vec<&'static str> = vec![
            "N","NNE","NE","ENE","E","ESE","SE","SSE",
            "S","SSW","SW","WSW","W","WNW","NW","NNW",
        ];
        let value = try!(
            self.try_read(&self.memory.wind_dir)
        );

        let index: usize = (value[0] >> 4) as usize;

        Ok(String::from(directions[index]))
    }

    pub fn wind_chill(&self) -> serial::Result<f32>
    {
        self.temperature(&self.memory.wind_chill)
    }

    pub fn rain_1h(&self) -> serial::Result<f32>
    {
        self.rain(&self.memory.rain_1h)
    }

    pub fn rain_24h(&self) -> serial::Result<f32>
    {
        self.rain(&self.memory.rain_24h)
    }

    pub fn rain_total(&self) -> serial::Result<f32>
    {
        self.rain(&self.memory.rain_total)
    }

    fn rain(&self, memory: &Memory) -> serial::Result<f32>
    {
        let value = try!(
            self.try_read(memory)
        );

        let low = (value[0] >> 4) as f32 / 10.0 + (value[0] & 0xF) as f32 / 100.0;
        let med = (value[1] >> 4) as f32 * 10.0 + (value[1] & 0xF) as f32;
        let high = (value[2] >> 4) as f32 * 1000.0 + (value[2] & 0xF) as f32 * 100.0;

        Ok(Self::round(low + med + high, 1))
    }

    pub fn pressure(&self) -> serial::Result<f32>
    {
        let value = try!(
            self.try_read(&self.memory.pressure)
        );

        let low = (value[0] >> 4) as f32 + (value[0] & 0xF) as f32 / 10.0;
        let med = (value[1] >> 4) as f32 * 100.0 + (value[1] & 0xF) as f32 * 10.0;
        let high = (value[2] & 0xF) as f32 * 1000.0;

        Ok(Self::round(low + med + high, 1))
    }

    fn try_read(&self, memory: &Memory) -> serial::Result<Vec<u8>>
    {
        for _ in 0..50 {
            match self.read(memory) {
                Ok(n) => return Ok(n),
                Err(_) => (),
            };
        }

        Err(
            serial::Error::new(serial::ErrorKind::Io(std::io::ErrorKind::Other), "Try read error")
        )
    }

    fn read(&self, memory: &Memory) -> serial::Result<Vec<u8>>
    {
        let mut response: Vec<u8> = Vec::with_capacity(memory.size);
        let mut buffer: [u8; 1] = [0; 1];
        let command = Self::encode_address(memory);

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

        for _ in 0..memory.size {
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
        let mut checksum: u32 = 0;

        for i in 0..response.len() {
            checksum += response[i] as u32;
        }

        checksum &= 0xFF;

        if checksum == answer as u32 {
            Ok(())
        }
        else {
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

    fn encode_address(memory: &Memory) -> Vec<u8>
    {
        let mut command: Vec<u8> = vec![];

        if memory.address == 0x06 {
            command = vec![0x06]
        }
        else {
            for i in 0..4 {
                let nibble = (memory.address >> (4 * (3 - i))) & 0x0F;
                command.push(0x82 + (nibble * 4) as u8);
            }

            command.push(
                std::cmp::min(0xC2 + memory.size * 4, 0xFE) as u8
            );
        }

        command
    }

    fn round(x: f32, n: u32) -> f32
    {
        let factor = 10u32.pow(n) as f32;
        let fract = (x.fract() * factor).round() / factor;

        x.trunc() + fract
    }
}

#[test]
fn test_address_encode()
{
    assert_eq!(Device::encode_address(&Memory {address: 0x06, size: 2}), &[0x06]);
    assert_eq!(Device::encode_address(&Memory {address: 0x346, size: 2}), &[130, 142, 146, 154, 202]);
}

#[test]
fn test_round()
{
    assert_eq!(Device::round(100.0, 2), 100.00);
    assert_eq!(Device::round(100.12345, 2), 100.12);
    assert_eq!(Device::round(-100.12345, 2), -100.12);
    assert_eq!(Device::round(100.12345, 5), 100.12345);
}
