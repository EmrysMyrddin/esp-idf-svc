//! OTA update example.
//!
//! In this example, we are checking for updates from a distant server hosting the latest firmware.
//! The hosting server is supposed to check if an update should be done or not by responding with
//! a 200 (OK, you should update) or 304 (Not Modified, you are up to date) status code.
//!
//! For this example to work, you need a OTA ready partition table.
//! You need at least 2 OTA partitions (see https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/kconfig.html#config-partition-table-type)
//!
//! The most common starting point for OTA is the "Factory app, two OTA definitions" layout (https://github.com/espressif/esp-idf/blob/master/components/partition_table/partitions_two_ota.csv).
//!
//! To use a custom partition table, download the CSV file of your choice (or create a custom one)
//! and use either use the `--partition-table` option of `espflash`, or set this option in your `espflash.toml` file.
//!
//! After a successful OTA update, you will need to reset the `otadata` partition. Otherwise, the ESP
//! will continue to boot on the second partition, while `cargo run` is flashing the first one by default.
//! To reset the `otadata` partition, add `--erase-parts otadata` to the runner command in `.cargo/config.toml`.

use anyhow::Context;
use embedded_svc::http::client::Client as HttpClient;
use esp_idf_svc::http::client::{EspHttpConnection, Method};
use esp_idf_svc::io;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_svc::{
    hal::peripherals::Peripherals,
    ota::{EspOta, SlotState},
};

use log::{error, info};

const VERSION: &str = "1.0.0"; // You can pull this from an environment variable at build time using env! macro.
const OTA_FIRMWARE_URI: &str = "http://your.domain/path/to/firmware";

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

mod http_status {
    pub const OK: u16 = 200;
    pub const NOT_MODIFIED: u16 = 304;
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    // Setup Wifi

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    connect_wifi(&mut wifi)?;

    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

    check_for_updates(&mut client)?;

    check_valid_state();

    Ok(())
}

fn check_valid_state() -> anyhow::Result<()> {
    let mut ota = EspOta::new()?;

    if ota.get_running_slot()?.state != SlotState::Valid {
        let is_app_valid = true;

        // Do the necessary checks to validate that your app is working as expected.
        // For example, you can contact your API to verify you still have access to it.

        if is_app_valid {
            ota.mark_running_slot_valid()?;
        } else {
            ota.mark_running_slot_invalid_and_reboot();
        }
    }

    Ok(())
}

fn check_for_updates(client: &mut HttpClient<EspHttpConnection>) -> anyhow::Result<()> {
    let request = client
        .request(
            Method::Get,
            OTA_FIRMWARE_URI,
            &[
                ("Accept", "application/octet-stream"),
                ("X-Esp32-Version", VERSION),
            ],
        )
        .context("failed to create update request")?;
    let response = request.submit().context("failed to send update request")?;

    if response.status() == http_status::NOT_MODIFIED {
        info!("Already up to date");
    } else if response.status() == http_status::OK {
        info!("An update is available, updating...");
        let mut ota = EspOta::new().context("failed to obtain OTA instance")?;

        let mut update = ota.initiate_update().context("failed to initiate update")?;

        match io::utils::copy(response, &mut update, &mut [0; 1024])
            .context("failed to download update")
        {
            Ok(_) => {
                info!("Update done. Restarting...");
                update.complete().context("failed to complete update")?;
                esp_idf_svc::hal::reset::restart();
            }
            Err(err) => {
                error!("Update failed: {err}");
                update.abort().context("failed to abort update")?;
            }
        };
    }

    Ok(())
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected");

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}
