//use esp_idf_sys as _;
use esp_idf_sys::*;
use esp32_nimble::{
    uuid128, BLEDevice, NimbleProperties, DescriptorProperties, NimbleSub,
    utilities::{mutex::Mutex, BleUuid},
    enums::AuthReq,
};
use crate::sensor;
use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, time::Duration};
use esp32_nimble::OnWriteArgs;
use std::os::unix::ffi::OsStrExt;
use zerocopy::IntoBytes;
use core::ffi::c_int;
use esp_idf_sys::esp_err_t;
use esp32_nimble::BLEAdvertisementData;
use heapless::spsc::Queue; 
use crate::espwifi::WifiManager; 

pub enum WifiCommand {
    Connect { ssid: String, pwd: String },
}

// Create queue somewhere
pub static WIFI_CMD_QUEUE: Mutex<Queue<WifiCommand, 4>> = Mutex::new(Queue::new());


extern "C" {
    fn esp_ble_gatt_set_local_mtu(mtu: c_int) -> esp_err_t;
}

fn on_ble_write(ssid: &str, pwd: &str, wifi_manager: &mut WifiManager) {
    if let Err(e) = wifi_manager.connect(ssid, pwd) {
        println!("❌ Wi-Fi connect error: {:?}", e);
    }
}

fn on_ble_read(wifi_manager: &WifiManager) -> &'static str {
    if wifi_manager.is_connected() {
        "Connected"
    } else {
        "Not connected"
    }
}

pub fn init_gas_ble_service() {
		
	let flag_notify = Arc::new(Mutex::new(false));
	
    esp_idf_sys::link_patches();
	
    let conn_handle = Arc::new(Mutex::new(None::<u16>));
    let conn_handle_clone = Arc::clone(&conn_handle);


    let ble_device = BLEDevice::take();
    BLEDevice::set_device_name("EnvSensor");
    ble_device.security().set_auth(AuthReq::Bond | AuthReq::Mitm);

    let server = ble_device.get_server();
    server.on_connect(|conn, _desc| {
        //let mtu = conn.att_mtu().unwrap_or(247);
        println!("Client connected");
    
    });

    server.on_disconnect(|_, _| println!("Client disconnected"));

    let service_uuid = uuid128!("749CFFF0-0000-1000-8000-00805F9B34FB");
    let service = server.create_service(service_uuid);


    // Respond to writes
    let notify_triggered = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let notify_enabled = Arc::new(AtomicBool::new(false));
    let notify_flag = notify_enabled.clone();
	
    // Writeable Characteristic (UUID3)
    let write_uuid = uuid128!("749CFFF1-0000-1000-8000-00805F9B34FB");
    let write_char = service.lock().create_characteristic(
        write_uuid,
        NimbleProperties::WRITE,
    );
	
	
	// Create descriptor
	let desc = write_char.lock().create_descriptor(
		uuid128!("00002901-0000-1000-8000-00805F9B34FB"),
		DescriptorProperties::READ,
	);

	// Set description text
	desc.lock().set_value(b"Notify flag (write to set 0[stop] & 1[start])");
	
	let flagnotify_clone = Arc::clone(&flag_notify);

    write_char.lock().on_write(move |args: &mut OnWriteArgs| {
         let data: &[u8] = args.recv_data();

         println!("Received write: {:?}", args.recv_data());

         if let Ok(s) = std::str::from_utf8(data) {
             println!("As string: {}", s);
         }

         let mut flag_val = flagnotify_clone.lock();
             

		 if data == [1u8]{
			*flag_val = true;
		 }else{
			*flag_val = false;
		 }
	});
	
	
	let ssid_val = Arc::new(Mutex::new(String::new()));
	let pwd_val = Arc::new(Mutex::new(String::new()));

	// SSID characteristic
	let write_ssiduuid = uuid128!("749CFFF5-0000-1000-8000-00805F9B34FB");
	let write_ssidchar = service.lock().create_characteristic(
		write_ssiduuid,
		NimbleProperties::WRITE,
	);
	
	

	{
		let ssid_val = Arc::clone(&ssid_val);
		let pwd_val = Arc::clone(&pwd_val);

		write_ssidchar.lock().on_write(move |args: &mut OnWriteArgs| {
			if let Ok(s) = core::str::from_utf8(args.recv_data()) {
				println!("Received SSID: {}", s);
				*ssid_val.lock() = s.to_string();

				let pwd = pwd_val.lock().clone();

				let mut q = WIFI_CMD_QUEUE.lock();
				q.enqueue(WifiCommand::Connect {
					ssid: s.to_string(),
					pwd,
				}).ok();
			}
		});
	}

	// PWD characteristic
	let write_pwduuid = uuid128!("749CFFF6-0000-1000-8000-00805F9B34FB");
	let write_pwdchar = service.lock().create_characteristic(
		write_pwduuid,
		NimbleProperties::WRITE,
	);

	{
		let ssid_val = Arc::clone(&ssid_val);
		let pwd_val = Arc::clone(&pwd_val);

		write_pwdchar.lock().on_write(move |args: &mut OnWriteArgs| {
			if let Ok(s) = core::str::from_utf8(args.recv_data()) {
				println!("Received PWD: {}", s);
				*pwd_val.lock() = s.to_string();

				let ssid = ssid_val.lock().clone();

				let mut q = WIFI_CMD_QUEUE.lock();
				q.enqueue(WifiCommand::Connect {
					ssid,
					pwd: s.to_string(),
				}).ok();
								
			}
		});
	}
		

    // Temperature Characteristic (UUID1)
    let temp_uuid = uuid128!("749CFFF2-0000-1000-8000-00805F9B34FB");
    let temp_char = service.lock().create_characteristic(
        temp_uuid,
        NimbleProperties::READ,
    );

    let temp_val = format!("{:.1}C", sensor::read_temperature());
    temp_char.lock().set_value(temp_val.as_bytes());


    temp_char.lock().on_read(|_conn_handle, val| {
        //let mtu = conn_handle.att_mtu().unwrap_or(247);   
        println!("Client read temp: {:?}", val);
    });
    
    let humidity_uuid = uuid128!("749CFFF3-0000-1000-8000-00805F9B34FB");
    let humidity_char = service.lock().create_characteristic(
        humidity_uuid,
        NimbleProperties::READ,
    );
	
    humidity_char.lock().set_value(b"60%");
    humidity_char.lock().on_read(move |_conn_handle, _val| {
        println!("Client read temp: {:?}", _val);
    });
	
    // Notify Characteristic (UUID2)
    let notify_uuid = uuid128!("749CFFF4-0000-1000-8000-00805F9B34FB");
    let notify_char = service.lock().create_characteristic(
        notify_uuid,
        NimbleProperties::NOTIFY,
    );

    let notify_flag_clone = notify_enabled.clone();
    let notify_char_clone = Arc::clone(&notify_char);

    notify_char.lock().on_subscribe(move |_conn_handle, _desc, subscription| {
        println!("Subscription: {:?}", subscription);
        if subscription.contains(NimbleSub::NOTIFY) {
           println!("Client subscribed");
           notify_flag_clone.store(true, Ordering::Relaxed);
        } else {
           println!("Client unsubscribed");
           notify_flag_clone.store(false, Ordering::Relaxed);
        }
    });


    let notify_flag_loop = notify_enabled.clone();
    let notify_char_loop = Arc::clone(&notify_char);

    thread::spawn(move || {
            loop {
                
                /*if notify_flag_loop.load(Ordering::Relaxed) {
                    let temp = sensor::read_temperature();
                    let hum = sensor::read_humidity();
                    let msg = format!("Temp: {:.1}C, Humidity: {:.1}%", temp, hum);

                    notify_char_loop.lock().set_value(msg.as_bytes());
                    notify_char_loop.lock().notify();
                    println!("Notified: {}", msg);
               }*/

				if *flag_notify.lock() {
					match sensor::read_uart() {
						  Ok(data) if !data.is_empty() => {
							  //notify_char_loop.lock().notify(); 
							  notify_char_loop.lock().set_value(data.as_slice());
							  //notify_char_loop.lock().set_value(data);
							  notify_char_loop.lock().notify();
							  //println!("Received UART Data: {:?}", data);
						  }
						  Ok(_) => {

						  }
						  Err(e) => {
							 println!("UART read error: {:?}", e);
						  }
					}
				}

           
                 thread::sleep(Duration::from_millis(1000));
           }
     });

    
	/*
	while let Some(cmd) = q.dequeue() {
					match cmd {
						WifiCommand::Connect { ssid, pwd } => {
							wifi_manager.lock().connect(&ssid, &pwd).ok();
						}
					}
				}
				*/
	
	
	let mut adv_data = BLEAdvertisementData::new();
	adv_data.name("EnvSensor");

	let adv = ble_device.get_advertising();
	let mut adv = adv.lock();

	// 👇 Pass `&mut adv_data` instead of `&adv_data`
	adv.set_data(&mut adv_data).unwrap();
	adv.start().unwrap();
	
	// Start advertising
    //ble_device.get_advertising().lock().start().unwrap();
    println!("BLE advertising started");
}
