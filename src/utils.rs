use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_svc::sys::EspError;

pub fn i2c_scan(i2c: &mut I2cDriver<'_>) {
    println!("\nI2C Scanner");
    println!("Scanning...");

    let mut n_devices = 0;

    for addr in 1..127 {
		let result = i2c.write(addr, &[], 1000);

        if result.is_ok() {
            println!("I2C device found at address 0x{:02X}", addr);
            n_devices += 1;
        } else if let Err(e) = result {
           
		   if e.code() == esp_idf_sys::ESP_FAIL {
				println!("Unknown I2C error at address 0x{:02X}", addr);
		   }
			
        }
    }

    if n_devices == 0 {
        println!("No I2C devices found\n");
    } else {
        println!("done\n");
    }
}

