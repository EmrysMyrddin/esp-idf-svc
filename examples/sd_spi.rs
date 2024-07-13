#[cfg(esp32)]
fn main() -> anyhow::Result<()> {
    use std::fs::{read_dir, File};
    use std::io::{Read, Seek, Write};

    use esp_idf_svc::fs::fat::FatFs;
    use esp_idf_svc::hal::gpio::AnyIOPin;
    use esp_idf_svc::hal::prelude::*;
    use esp_idf_svc::hal::sd::{spi::SdSpiHostDriver, SdCardDriver};
    use esp_idf_svc::hal::spi::{config::DriverConfig, Dma, SpiDriver};
    use esp_idf_svc::io::vfs::MountedFatFs;
    use esp_idf_svc::log::EspLogger;

    esp_idf_svc::sys::link_patches();

    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let pins = peripherals.pins;

    let spi_driver = SpiDriver::new(
        peripherals.spi3,
        pins.gpio18,
        pins.gpio23,
        Some(pins.gpio19),
        &DriverConfig::default().dma(Dma::Auto(4096)),
    )?;

    let sd_card_driver = SdCardDriver::new_spi(SdSpiHostDriver::new(
        spi_driver,
        Some(pins.gpio5),
        AnyIOPin::none(),
        AnyIOPin::none(),
        AnyIOPin::none(),
        #[cfg(not(any(
            esp_idf_version_major = "4",
            all(esp_idf_version_major = "5", esp_idf_version_minor = "0"),
            all(esp_idf_version_major = "5", esp_idf_version_minor = "1"),
        )))] // For ESP-IDF v5.2 and later
        None,
    )?)?;

    // Keep it around or else it will be dropped and unmounted
    let _mounted_fat_fs = MountedFatFs::mount(FatFs::new_sdcard(0, sd_card_driver)?, "/sdspi", 4)?;

    let content = b"Hello, world!";

    {
        let mut file = File::create("/sdspi/test.txt")?;

        file.write_all(content).expect("Write failed");

        file.seek(std::io::SeekFrom::Start(0)).expect("Seek failed");
    }

    {
        let mut file = File::open("/sdspi/test.txt")?;

        let mut file_content = String::new();

        file.read_to_string(&mut file_content).expect("Read failed");

        assert_eq!(file_content.as_bytes(), content);
    }

    {
        let directory = read_dir("/sdspi")?;

        for entry in directory {
            log::info!("Entry: {:?}", entry?.file_name());
        }
    }

    Ok(())
}

#[cfg(not(esp32))]
fn main() {
    use esp_idf_svc::{self as _};

    panic!("This example is configured for esp32, please adjust pins to your module");
}
