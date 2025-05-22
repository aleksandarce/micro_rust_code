#![no_std]
#![no_main]

use cyw43_pio::PioSpi;
use embassy_rp::gpio::{Level, Output};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use defmt::*;

use static_cell::StaticCell;
use embassy_rp::bind_interrupts;
use embassy_rp::i2c::{I2c, InterruptHandler};
use embedded_hal_async::i2c::{Error, I2c as _};
use embassy_rp::peripherals::*;
use embassy_net::{Config, Stack, StackResources, Ipv4Address, IpAddress, Ipv4Cidr, StaticConfigV4};
use embassy_lab_utils::{init_wifi, init_network_stack};
use cyw43::*;
use embassy_net::tcp::TcpSocket;
use embassy_rp::pio::Pio;
use embassy_rp::spi::Spi;
use cyw43_pio::*;

// Import interrupts definition module
mod irqs;

// bind_interrupts!(struct Irqs {
//     PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
// });

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Step 1: Init Wi-Fi driver
    let (net_device, mut control) = init_wifi!(&spawner, p).await;

    control.set_power_management(PowerManagementMode::None).await;

    // Step 2: Start Open Access Point (NO PASSWORD)
    control
        .start_ap_open("Bilge", 6) // (SSID, channel)
        .await;
    info!("✅ Access Point started: SSID = PICO_AP");

    // Step 3: Static IP configuration (192.168.4.1)
    let ip = Ipv4Address::new(192, 168, 4, 1);
    let config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(ip, 24),
        gateway: Some(ip),
        dns_servers: heapless::Vec::new(),
    });

    static RESOURCES: StaticCell<StackResources<1>> = StaticCell::new();
    let _stack = init_network_stack(&spawner, net_device, &RESOURCES, config);

    info!("Static IP set to {}", ip);
    info!("Connect your PC to SSID 'PICO_AP' and set IP to 192.168.4.2");

    loop {
        Timer::after_secs(10).await;
    }
}

// #[embassy_executor::main]
// async fn main(spawner: Spawner) {
//     let peripherals = embassy_rp::init(Default::default());
//
//     let (net_device, mut control) = init_wifi!(&spawner, peripherals).await;
//     control.set_power_management(PowerManagementMode::None).await;
//
//     {
//         let mut scanner = control.scan(Default::default()).await;
//         while let Some(bss) = scanner.next().await {
//             if let Ok(ssid) = core::str::from_utf8(&bss.ssid) {
//                 info!("Found network: {}", ssid);
//             }
//         }
//     }
//
//     let ssid = "UPB-Guest";
//     let options = JoinOptions::new_open();
//
//
//     match control.join(ssid, options).await {
//         Ok(_) => info!("Connected to open Wi-Fi network!"),
//         Err(e) => {
//             warn!("Failed to connect to Wi-Fi");
//             return;
//         }
//     }
//
//     static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
//
//     let stack = init_network_stack(
//         &spawner,
//         net_device,
//         &RESOURCES,
//         Config::dhcpv4(Default::default()),
//     );
//
//
//     loop {
//         if let Some(config) = stack.config_v4() {
//             let ip = config.address.address();
//             info!("🎉 Got IP via DHCP: {}", ip);
//             break;
//         }
//         Timer::after_millis(500).await;
//     }
//
//     loop {
//         Timer::after_secs(10).await;
//     }
// }

// #[embassy_executor::task]
// async fn wifi_task(runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>) -> ! {
//     runner.run().await
// }

// bind_interrupts!(struct Irqs {
//     I2C1_IRQ => InterruptHandler<I2C1>;
// });
// 
// const REG_CTRL_MEAS: u8 = 0xF4;
// const REG_TEMP_MSB: u8 = 0xFA;
// const REG_CALIB_START: u8 = 0x88;
// const EEPROM_ADDR: u8 = 0x50;
// const EEPROM_TEMP_ADDR: u16 = 0xACDC;
// 
// #[embassy_executor::main]
// async fn main(_spawner: Spawner) {
//     let p = embassy_rp::init(Default::default());
// 
//     let scl = p.PIN_19;
//     let sda = p.PIN_18;
//     let mut i2c = I2c::new_async(p.I2C1, scl, sda, Irqs, Config::default());
// 
//     let bmp_address: u8 = 0x76;
// 
//     if let Err(e) = i2c.write(bmp_address, &[REG_CTRL_MEAS, 0x43]).await {
//         warn!("Failed to configure sensor: {:?}", e);
//         return;
//     }
// 
//     let mut calib_buf = [0u8; 6];
//     if let Err(e) = i2c.write_read(bmp_address, &[REG_CALIB_START], &mut calib_buf).await {
//         warn!("Failed to read calibration data: {:?}", e);
//         return;
//     }
// 
//     let dig_t1: u16 = ((calib_buf[1] as u16) << 8) | (calib_buf[0] as u16);
//     let dig_t2: i16 = ((calib_buf[3] as i16) << 8) | (calib_buf[2] as i16);
//     let dig_t3: i16 = ((calib_buf[5] as i16) << 8) | (calib_buf[4] as i16);
// 
//     info!("Calibration constants: T1 = {}, T2 = {}, T3 = {}", dig_t1, dig_t2, dig_t3);
// 
//     let mut temp_buf = [0u8; 4];
//     let addr_bytes = EEPROM_TEMP_ADDR.to_be_bytes();
// 
//     match i2c.write_read(EEPROM_ADDR, &addr_bytes, &mut temp_buf).await {
//         Ok(_) => {
//             let saved_temp = i32::from_le_bytes(temp_buf);
//             info!(
//                 "Last saved temperature from EEPROM: {}.{}°C",
//                 saved_temp / 100,
//                 saved_temp.abs() % 100
//             );
//         }
//         Err(e) => {
//             warn!("Failed to read saved temperature from EEPROM: {:?}", e);
//         }
//     }
// 
//     loop {
//         let mut buf = [0u8; 3];
//         if let Err(e) = i2c.write_read(bmp_address, &[REG_TEMP_MSB], &mut buf).await {
//             warn!("Failed to read temperature: {:?}", e);
//             Timer::after_secs(2).await;
//             continue;
//         }
// 
//         let msb = buf[0];
//         let lsb = buf[1];
//         let xlsb = buf[2];
// 
//         let raw_temp: u32 =
//             ((msb as u32) << 12) | ((lsb as u32) << 4) | ((xlsb as u32) >> 4);
//         let var1 = (((raw_temp as i32 >> 3) - ((dig_t1 as i32) << 1)) * (dig_t2 as i32)) >> 11;
//         let var2 = (((((raw_temp as i32 >> 4) - (dig_t1 as i32)) * ((raw_temp as i32 >> 4) - (dig_t1 as i32))) >> 12)
//             * (dig_t3 as i32))
//             >> 14;
//         let t_fine = var1 + var2;
//         let actual_temp = (t_fine * 5 + 128) >> 8;
// 
//         info!(
//             "Temperature {}.{}°C",
//             actual_temp / 100,
//             actual_temp.abs() % 100
//         );
// 
//         let temp_bytes = actual_temp.to_le_bytes();
//         let mut write_data = [0u8; 6];
//         write_data[0..2].copy_from_slice(&EEPROM_TEMP_ADDR.to_be_bytes());
//         write_data[2..6].copy_from_slice(&temp_bytes);
// 
//         if let Err(e) = i2c.write(EEPROM_ADDR, &write_data).await {
//             warn!("Failed to write to EEPROM: {:?}", e);
//         }
// 
//         Timer::after_secs(2).await;
//     }
// }
