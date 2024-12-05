#![warn(warnings)]

use std::cell::RefCell;
use std::thread::sleep;
use std::time::Duration;

pub struct Device {
    port: RefCell<Box<dyn serialport::SerialPort>>,
    memory: MemoryMap,
}

#[derive(serde_derive::Serialize)]
pub struct Data {
    temperature_indoor: f32,
    temperature_outdoor: f32,
    dewpoint: f32,
    humidity_indoor: u32,
    humidity_outdoor: u32,
    wind_speed: f32,
    wind_dir: f32,
    wind_direction: String,
    wind_chill: f32,
    rain_1h: f32,
    rain_24h: f32,
    rain_total: f32,
    pressure: f32,
    tendency: String,
    forecast: String,
}

struct MemoryMap {
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
    tendency: Memory,
}

struct Memory {
    address: u32,
    size: usize,
}

impl Device {
    pub fn new(device: String) -> serialport::Result<Device> {
        let memory = MemoryMap {
            temperature_indoor: Memory {
                address: 0x346,
                size: 2,
            },
            temperature_outdoor: Memory {
                address: 0x373,
                size: 2,
            },
            dewpoint: Memory {
                address: 0x3CE,
                size: 2,
            },
            humidity_indoor: Memory {
                address: 0x3FB,
                size: 1,
            },
            humidity_outdoor: Memory {
                address: 0x419,
                size: 1,
            },
            wind_speed: Memory {
                address: 0x529,
                size: 3,
            },
            wind_dir: Memory {
                address: 0x52C,
                size: 1,
            },
            wind_chill: Memory {
                address: 0x3A0,
                size: 2,
            },
            rain_1h: Memory {
                address: 0x4B4,
                size: 3,
            },
            rain_24h: Memory {
                address: 0x497,
                size: 3,
            },
            rain_total: Memory {
                address: 0x4D2,
                size: 3,
            },
            pressure: Memory {
                address: 0x5E2,
                size: 3,
            },
            tendency: Memory {
                address: 0x26B,
                size: 1,
            },
        };

        let device = Device {
            port: Self::open(device)?.into(),
            memory,
        };

        Ok(device)
    }

    fn open(device: String) -> serialport::Result<Box<dyn serialport::SerialPort>> {
        let mut port = serialport::new(&device, 2_400)
            .data_bits(serialport::DataBits::Eight)
            .flow_control(serialport::FlowControl::None)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .open()?;

        Self::setup(&mut port)?;

        Ok(port)
    }

    fn setup(port: &mut Box<dyn serialport::SerialPort>) -> serialport::Result<()> {
        port.write_request_to_send(true)?;
        port.write_data_terminal_ready(false)?;

        Ok(())
    }

    pub fn read_all(&self) -> serialport::Result<Data> {
        Ok(Data {
            temperature_indoor: self.temperature_indoor()?,
            temperature_outdoor: self.temperature_outdoor()?,
            dewpoint: self.dewpoint()?,
            humidity_indoor: self.humidity_indoor()?,
            humidity_outdoor: self.humidity_outdoor()?,
            wind_speed: self.wind_speed()?,
            wind_dir: self.wind_dir()?,
            wind_direction: self.wind_direction()?,
            wind_chill: self.wind_chill()?,
            rain_1h: self.rain_1h()?,
            rain_24h: self.rain_24h()?,
            rain_total: self.rain_total()?,
            pressure: self.pressure()?,
            tendency: self.tendency()?,
            forecast: self.forecast()?,
        })
    }

    pub fn temperature_indoor(&self) -> serialport::Result<f32> {
        self.temperature(&self.memory.temperature_indoor)
    }

    pub fn temperature_outdoor(&self) -> serialport::Result<f32> {
        self.temperature(&self.memory.temperature_outdoor)
    }

    pub fn dewpoint(&self) -> serialport::Result<f32> {
        self.temperature(&self.memory.dewpoint)
    }

    fn temperature(&self, memory: &Memory) -> serialport::Result<f32> {
        let value = self.try_read(memory)?;

        let low = (value[0] >> 4) as f32 / 10.0 + (value[0] & 0xF) as f32 / 100.0;
        let high = (value[1] >> 4) as f32 * 10.0 + (value[1] & 0xF) as f32;

        Ok(Self::round(high + low - 30.0, 1))
    }

    pub fn humidity_indoor(&self) -> serialport::Result<u32> {
        self.humidity(&self.memory.humidity_indoor)
    }

    pub fn humidity_outdoor(&self) -> serialport::Result<u32> {
        self.humidity(&self.memory.humidity_outdoor)
    }

    fn humidity(&self, memory: &Memory) -> serialport::Result<u32> {
        let value = self.try_read(memory)?;

        let low = (value[0] >> 4) as u32 * 10 + (value[0] & 0xF) as u32;

        Ok(low)
    }

    pub fn wind_speed(&self) -> serialport::Result<f32> {
        let value = self.try_read(&self.memory.wind_speed)?;

        Ok(((((value[1] & 0xF) as u16) << 8) as f32 + value[0] as f32) / 10.0)
    }

    pub fn wind_dir(&self) -> serialport::Result<f32> {
        let value = self.try_read(&self.memory.wind_dir)?;

        let low = (value[0] >> 4) as f32;

        Ok(Self::round(low * 22.5, 1))
    }

    pub fn wind_direction(&self) -> serialport::Result<String> {
        let directions: Vec<&'static str> = vec![
            "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
            "NW", "NNW",
        ];
        let value = self.try_read(&self.memory.wind_dir)?;

        let index: usize = (value[0] >> 4) as usize;

        Ok(String::from(directions[index]))
    }

    pub fn wind_chill(&self) -> serialport::Result<f32> {
        self.temperature(&self.memory.wind_chill)
    }

    pub fn rain_1h(&self) -> serialport::Result<f32> {
        self.rain(&self.memory.rain_1h)
    }

    pub fn rain_24h(&self) -> serialport::Result<f32> {
        self.rain(&self.memory.rain_24h)
    }

    pub fn rain_total(&self) -> serialport::Result<f32> {
        self.rain(&self.memory.rain_total)
    }

    fn rain(&self, memory: &Memory) -> serialport::Result<f32> {
        let value = self.try_read(memory)?;

        let low = (value[0] >> 4) as f32 / 10.0 + (value[0] & 0xF) as f32 / 100.0;
        let med = (value[1] >> 4) as f32 * 10.0 + (value[1] & 0xF) as f32;
        let high = (value[2] >> 4) as f32 * 1000.0 + (value[2] & 0xF) as f32 * 100.0;

        Ok(Self::round(low + med + high, 1))
    }

    pub fn pressure(&self) -> serialport::Result<f32> {
        let value = self.try_read(&self.memory.pressure)?;

        let low = (value[0] >> 4) as f32 + (value[0] & 0xF) as f32 / 10.0;
        let med = (value[1] >> 4) as f32 * 100.0 + (value[1] & 0xF) as f32 * 10.0;
        let high = (value[2] & 0xF) as f32 * 1000.0;

        Ok(Self::round(low + med + high, 1))
    }

    pub fn tendency(&self) -> serialport::Result<String> {
        let tendencies: Vec<&'static str> = vec!["Steady", "Rising", "Falling"];

        let value = self.try_read(&self.memory.tendency)?;

        let index = (value[0] >> 4) as usize;

        Ok(String::from(tendencies[index]))
    }

    pub fn forecast(&self) -> serialport::Result<String> {
        let forecasts: Vec<&'static str> = vec!["Rainy", "Cloudy", "Sunny"];

        let value = self.try_read(&self.memory.tendency)?;

        let index = (value[0] & 0xF) as usize;

        Ok(String::from(forecasts[index]))
    }

    fn try_read(&self, memory: &Memory) -> serialport::Result<Vec<u8>> {
        for _ in 0..50 {
            if let Ok(n) = self.read(memory) {
                return Ok(n);
            }
        }

        Err(serialport::Error::new(
            serialport::ErrorKind::Io(std::io::ErrorKind::Other),
            "Try read error",
        ))
    }

    fn read(&self, memory: &Memory) -> serialport::Result<Vec<u8>> {
        let mut response: Vec<u8> = Vec::with_capacity(memory.size);
        let mut buffer: [u8; 1] = [0; 1];
        let command = Self::encode_address(memory);

        self.reset()?;

        for (i, c) in command.iter().enumerate().take(5) {
            self.port.borrow_mut().write_all(&[*c])?;
            self.port.borrow_mut().read_exact(&mut buffer[..])?;
            Self::check(*c, i, buffer[0])?;
        }

        for _ in 0..memory.size {
            self.port.borrow_mut().read_exact(&mut buffer[..])?;

            response.push(buffer[0]);
        }

        self.port.borrow_mut().read_exact(&mut buffer[..])?;

        Self::check_data(buffer[0], response.clone())?;

        Ok(response)
    }

    fn check(command: u8, sequence: usize, answer: u8) -> serialport::Result<()> {
        let checksum = if sequence < 4 {
            (sequence as u8) * 16 + (command - 0x82) / 4
        } else {
            0x30 + (command - 0xC2) / 4
        };

        if checksum == answer {
            Ok(())
        } else {
            Err(serialport::Error::new(
                serialport::ErrorKind::Io(std::io::ErrorKind::Other),
                "Check error",
            ))
        }
    }

    fn check_data(answer: u8, response: Vec<u8>) -> serialport::Result<()> {
        let mut checksum: u32 = 0;

        for r in &response {
            checksum += *r as u32;
        }

        checksum &= 0xFF;

        if checksum == answer as u32 {
            Ok(())
        } else {
            Err(serialport::Error::new(
                serialport::ErrorKind::Io(std::io::ErrorKind::Other),
                "Check data error",
            ))
        }
    }

    fn reset(&self) -> serialport::Result<()> {
        let mut buffer: [u8; 1] = [0; 1];

        for x in 0..100 {
            self.port.borrow_mut().flush()?;
            self.port.borrow_mut().write_all(&[0x06])?;

            loop {
                self.port
                    .borrow_mut()
                    .read_exact(&mut buffer[..])
                    .map_err(|_| {
                        serialport::Error::new(
                            serialport::ErrorKind::Io(std::io::ErrorKind::Other),
                            "reset failed",
                        )
                    })?;

                if buffer[0] == 1 {
                    continue;
                } else if buffer[0] == 2 {
                    return Ok(());
                } else {
                    break;
                }
            }

            sleep(Duration::from_millis(100 * x));
        }

        Ok(())
    }

    fn encode_address(memory: &Memory) -> Vec<u8> {
        let mut command: Vec<u8> = vec![];

        if memory.address == 0x06 {
            command = vec![0x06]
        } else {
            for i in 0..4 {
                let nibble = (memory.address >> (4 * (3 - i))) & 0x0F;
                command.push(0x82 + (nibble * 4) as u8);
            }

            command.push(std::cmp::min(0xC2 + memory.size * 4, 0xFE) as u8);
        }

        command
    }

    fn round(x: f32, n: u32) -> f32 {
        let factor = 10u32.pow(n) as f32;
        let fract = (x.fract() * factor).round() / factor;

        x.trunc() + fract
    }
}

#[test]
fn test_address_encode() {
    assert_eq!(
        Device::encode_address(&Memory {
            address: 0x06,
            size: 2
        }),
        &[0x06]
    );
    assert_eq!(
        Device::encode_address(&Memory {
            address: 0x346,
            size: 2
        }),
        &[130, 142, 146, 154, 202]
    );
}

#[test]
fn test_round() {
    assert_eq!(Device::round(100.0, 2), 100.00);
    assert_eq!(Device::round(100.12345, 2), 100.12);
    assert_eq!(Device::round(-100.12345, 2), -100.12);
    assert_eq!(Device::round(100.12345, 5), 100.12345);
}
