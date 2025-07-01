use esp_idf_svc::wifi::*;
use anyhow::Result;
use esp_idf_svc::wifi::{EspWifi, BlockingWifi, ClientConfiguration};
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use embedded_svc::wifi::Configuration;
use esp_idf_hal::modem::Modem;
use heapless::String;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

pub struct WifiManager<'a> {
    //driver: Option<EspWifi<'a>>,
    blocking: Option<BlockingWifi<EspWifi<'a>>>,
    nvs: EspDefaultNvs,
    connected: bool,
}

fn save_wifi_credentials(nvs: &mut EspDefaultNvs, ssid: &str, pwd: &str) -> anyhow::Result<()> {
    nvs.set_str("wifi_ssid", ssid)?;
    nvs.set_str("wifi_pwd", pwd)?;
    Ok(())
}

fn load_wifi_credentials(nvs: &EspDefaultNvs) -> Option<(String<32>, String<64>)> {
    
    let mut ssid_buf = [0u8; 32];
    let mut pwd_buf = [0u8; 64];

    let ssid = nvs.get_str("wifi_ssid", &mut ssid_buf).ok()??;
    let pwd = nvs.get_str("wifi_pwd", &mut pwd_buf).ok()??;

    let mut ssid_buf = String::new();
    let mut pwd_buf = String::new();
    ssid_buf.push_str(&ssid).ok()?;
    pwd_buf.push_str(&pwd).ok()?;

    Some((ssid_buf, pwd_buf))
}

impl<'a> WifiManager<'a> {
    pub fn new(modem: Modem) -> anyhow::Result<Self> {
        esp_idf_sys::link_patches();
        let sys_loop = EspSystemEventLoop::take()?;

        let nvs_partition = EspDefaultNvsPartition::take()?; 
        //let nvs = EspDefaultNvs::new(partition, "wifi", true)?;
        let wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs_partition.clone()))?; 
        //let wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
        let blocking = BlockingWifi::wrap(wifi, sys_loop)?;
        
        let nvs = EspDefaultNvs::new(nvs_partition, "wifi", true)?;

        Ok(Self {
            //driver: Some(wifi),
            blocking: Some(blocking),
            nvs,
            connected: false,
        })
    }

    pub fn connect(&mut self, ssid: &str, pwd: &str) -> anyhow::Result<()> {
        let mut ssid_buf = String::<32>::new();
        let mut pwd_buf = String::<64>::new();
        ssid_buf.push_str(ssid).map_err(|_| anyhow::anyhow!("SSID too long"))?;
        pwd_buf.push_str(pwd).map_err(|_| anyhow::anyhow!("Password too long"))?;

        if let Some(blocking) = &mut self.blocking {
            if blocking.is_connected()? {
                blocking.disconnect()?;
            }

            blocking.set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: ssid_buf.clone(),
                password: pwd_buf.clone(),
                ..Default::default()
            }))?;
            blocking.start()?;
            blocking.connect()?;
            blocking.wait_netif_up()?;

            save_wifi_credentials(&mut self.nvs, ssid, pwd)?;
            self.connected = true;
            println!("✅ Connected to Wi-Fi: {}", ssid);
        }

        Ok(())
    }

    pub fn try_connect_saved(&mut self) -> anyhow::Result<()> {
        if let Some((ssid, pwd)) = load_wifi_credentials(&self.nvs) {
            println!("🔄 Connecting to saved Wi-Fi: {}", ssid.as_str());
            self.connect(ssid.as_str(), pwd.as_str())?;
        } else {
            println!("⚠ No saved Wi-Fi credentials found");
        }
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}


/// Sends a message via MQTT
pub fn send_msg() -> Result<()> {
    let (mut client, _event_loop) = EspMqttClient::new(
        "mqtt://broker.emqx.io",
        &MqttClientConfiguration::default(),
    )?;

    client.publish("esp32/status", QoS::AtMostOnce, false, b"Hello from ESP32!")?;
    println!("📡 Message sent via MQTT");

    Ok(())
}
