mod bleserver;
mod sensor;
use std::thread;
use std::time::Duration;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::gpio::*;
use esp_idf_hal::i2c::*;
use esp_idf_hal::delay::FreeRtos;
mod gpio;
mod espwifi;
mod utils;
use espwifi::{WifiManager, send_msg};
use std::sync::Arc;
use esp32_nimble::utilities::mutex::Mutex;
use crate::utils::i2c_scan;

fn main() {

    println!("Starting BLE server...");

    let mut peripherals = Peripherals::take().unwrap();

    let pins = peripherals.pins;
    let uart0 = peripherals.uart0;
    let i2c0 = peripherals.i2c0;
    let modem = peripherals.modem;
		
	/* Scan for i2C address*/
	let sda = pins.gpio21; // adjust as needed
    let scl = pins.gpio22;

    //let config = I2cConfig::new().baudrate(100.kHz().into());
	//let mut i2c = I2cDriver::new( i2c0,  sda,  scl, &config).unwrap();
	//i2c_scan(&mut i2c);
	
    let wm = WifiManager::new(modem).expect("Failed to create WifiManager");
    let wifi_manager = Arc::new(Mutex::new(wm));

    sensor::init_uart(uart0, pins.gpio17, pins.gpio16);
    bleserver::init_gas_ble_service();
	
	//Read from i2c	
    match sensor::read_from_i2c(
        i2c0,
        sda,
        scl,
    ) {
        Ok(data) => println!("I2C Read Result: 0x{:02X}", data),
        Err(e) => eprintln!("I2C Read Error: {}", e),
    }
	  
	wifi_manager.lock().try_connect_saved();
    //Wifi SSID and PWD changes read and reset wifi	
    thread::spawn(move || {
		loop {
			let mut q = bleserver::WIFI_CMD_QUEUE.lock();
			/*if q.is_empty() {
				println!("⚠ WIFI_CMD_QUEUE is empty — nothing to process");
			}*/
			while let Some(cmd) = q.dequeue() {
				match cmd {
					bleserver::WifiCommand::Connect { ssid, pwd } => {
						println!("✅ Received command: SSID='{}' PWD='{}'", ssid, pwd);
						let result = wifi_manager.lock().connect(&ssid, &pwd);
						match result {
							Ok(_) => { println!("🌐 Connect command issued successfully");let _ = send_msg(); }
							Err(e) => println!("❌ Connect failed: {:?}", e),
						}
					}
				}
			}
			
			FreeRtos::delay_ms(100); // small sleep to avoid tight loop
			
		}
    });
    
	println!("✅ LED blinking loop");
    
	
	// 💡 LED blink thread
     // Access GPIO2
    //let mut led = GpioOut::new(pins.gpio2).unwrap();
	let led = PinDriver::output(pins.gpio2).unwrap();
    let led = Arc::new(Mutex::new(led));
	
    let led = Arc::clone(&led);
    thread::spawn(move || {
        loop {
			//println!("✅ LED blinking started");
            {
                let mut led = led.lock();
                led.set_high().unwrap();
				//println!("🔆 LED ON");
            }
            FreeRtos::delay_ms(500);

            {
                let mut led = led.lock();
                led.set_low().unwrap();
				//println!("⚫ LED OFF");
            }
            FreeRtos::delay_ms(500);
        }
    });
    
}
