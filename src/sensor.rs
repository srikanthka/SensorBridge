use rand::Rng;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::uart::*;
use esp_idf_hal::gpio::*;
use esp_idf_hal::i2c::{I2C0,I2cDriver};
use std::time::Duration;
use std::io::{Read};
use std::thread;
use crate::sensor::config::Config;
use esp_idf_sys::configTICK_RATE_HZ;
use esp_idf_hal::uart::UartDriver;
use once_cell::sync::OnceCell;
use esp32_nimble::utilities::mutex::Mutex;
use esp_idf_hal::sys::EspError;

static UART: OnceCell<Mutex<UartDriver>> = OnceCell::new();


pub fn read_temperature() -> f32 {
        let mut rng = rand::thread_rng();
            20.0 + rng.gen_range(0.0..10.0)
}

pub fn read_humidity() -> f32 {
       
        let mut rng = rand::thread_rng();
            40.0 + rng.gen_range(0.0..30.0)
}

pub fn read_from_i2c(i2c0: I2C0, sda: Gpio21, scl: Gpio22) -> Result<u8, &'static str> {
    let mut i2c = I2cDriver::new(i2c0, sda, scl, &Default::default())
        .map_err(|_| "Failed to create I2C driver")?;

    let address = 0x68; // Replace with your sensor’s I2C address
    let register = [0x75]; // Replace with the register you want to read from
    let mut buffer = [0u8; 1];

    i2c.write_read(address, &register, &mut buffer, 100)
        .map_err(|_| "I2C read failed")?;

    Ok(buffer[0])
}


pub fn init_uart(uart0:UART0, tx:Gpio17,rx:Gpio16) {
        //let peripherals = Peripherals::take().expect("Peripherals already taken");

        let config = Config::default();

        let uart = UartDriver::new(
                   uart0,
                   tx,
                   rx,
                   Option::<esp_idf_hal::gpio::AnyIOPin>::None,
                   Option::<esp_idf_hal::gpio::AnyIOPin>::None,
                   &config,
                   )
                   .expect("Failed to initialize UART");

        //UART.set(Mutex::new(uart)).expect("UART already initialized");
        UART.set(Mutex::new(uart)).unwrap_or_else(|_| panic!("UART already initialized"));
}


pub fn read_uart() -> Result<Vec<u8>, esp_idf_hal::sys::EspError> {
        let uart = UART.get().expect("UART not initialized").lock();

        let mut buffer = [0u8; 128];
        let timeout_ms = 1000u32;
        let timeout_ticks = timeout_ms * configTICK_RATE_HZ / 1000;

        match uart.read(&mut buffer, timeout_ticks) {
              Ok(n) => { 
                  if n > 0 {
                      //println!("Received UART Data: {:?}", &buffer[..n]);
                      Ok(buffer[..n].to_vec())
                  }else{
                     Err(EspError::from(esp_idf_sys::ESP_ERR_TIMEOUT).unwrap())
                  }
              }
              Err(e) => {
                 if e.code() != esp_idf_sys::ESP_ERR_TIMEOUT {
                    println!("UART read error: {:?}", e);
                 }
                 Err(e)
             }
       }
}




