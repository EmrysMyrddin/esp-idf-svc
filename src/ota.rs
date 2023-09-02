//! Over The Air Updates (OTA)
//!
//! The OTA update mechanism allows a device to update itself based on data
//! received while the normal firmware is running (for example, over Wi-Fi or
//! Bluetooth.)

use core::cmp::min;
use core::fmt::Write;
use core::marker::PhantomData;
use core::mem;
use core::ptr;

use ::log::*;
use embedded_svc::ota::Slot;

use embedded_svc::io;
use embedded_svc::ota;

use crate::sys::*;

use crate::io::EspIOError;
use crate::private::{common::*, cstr::*, mutex};

static TAKEN: mutex::Mutex<bool> = mutex::Mutex::wrap(mutex::RawMutex::new(), false);

impl From<Newtype<&esp_app_desc_t>> for ota::FirmwareInfo {
    fn from(app_desc: Newtype<&esp_app_desc_t>) -> Self {
        let app_desc = app_desc.0;

        let mut result = Self {
            version: unsafe { from_cstr_ptr(&app_desc.version as *const _).into() },
            signature: Some(heapless::Vec::from_slice(&app_desc.app_elf_sha256).unwrap()),
            released: "".into(),
            description: Some(unsafe { from_cstr_ptr(&app_desc.project_name as *const _).into() }),
            download_id: None,
        };

        write!(
            &mut result.released,
            "{} {}",
            unsafe { from_cstr_ptr(&app_desc.date as *const _) },
            unsafe { from_cstr_ptr(&app_desc.time as *const _) }
        )
        .unwrap();

        result
    }
}

pub struct EspFirmwareInfoLoader(heapless::Vec<u8, 512>);

impl EspFirmwareInfoLoader {
    pub fn new() -> Self {
        Self(heapless::Vec::new())
    }
}

impl Default for EspFirmwareInfoLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl io::ErrorType for EspFirmwareInfoLoader {
    type Error = EspIOError;
}

impl ota::FirmwareInfoLoader for EspFirmwareInfoLoader {
    fn load(&mut self, buf: &[u8]) -> Result<ota::LoadResult, Self::Error> {
        if !self.is_loaded() {
            let remaining = self.0.capacity() - self.0.len();
            if remaining > 0 {
                self.0
                    .extend_from_slice(&buf[..min(buf.len(), remaining)])
                    .unwrap();
            }
        }

        Ok(if self.is_loaded() {
            ota::LoadResult::Loaded
        } else {
            ota::LoadResult::LoadMore
        })
    }

    fn is_loaded(&self) -> bool {
        self.0.len()
            >= mem::size_of::<esp_image_header_t>()
                + mem::size_of::<esp_image_segment_header_t>()
                + mem::size_of::<esp_app_desc_t>()
    }

    fn get_info(&self) -> Result<ota::FirmwareInfo, Self::Error> {
        if self.is_loaded() {
            let app_desc_slice = &self.0[mem::size_of::<esp_image_header_t>()
                + mem::size_of::<esp_image_segment_header_t>()
                ..mem::size_of::<esp_image_header_t>()
                    + mem::size_of::<esp_image_segment_header_t>()
                    + mem::size_of::<esp_app_desc_t>()];

            let app_desc = unsafe {
                (app_desc_slice.as_ptr() as *const esp_app_desc_t)
                    .as_ref()
                    .unwrap()
            };
            Ok(Newtype(app_desc).into())
        } else {
            Err(EspError::from_infallible::<ESP_ERR_INVALID_SIZE>().into())
        }
    }
}

#[derive(Debug)]
pub struct EspOtaUpdate<'a> {
    update_partition: *const esp_partition_t,
    update_handle: esp_ota_handle_t,
    _data: PhantomData<&'a mut ()>,
}

impl<'a> EspOtaUpdate<'a> {
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, EspError> {
        self.check_write()?;

        esp!(unsafe { esp_ota_write(self.update_handle, buf.as_ptr() as _, buf.len() as _) })?;

        Ok(buf.len())
    }

    pub fn flush(&mut self) -> Result<(), EspError> {
        self.check_write()?;

        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), EspError> {
        self.check_write()?;

        esp!(unsafe { esp_ota_end(self.update_handle) })?;
        esp!(unsafe { esp_ota_set_boot_partition(self.update_partition) })?;

        self.update_partition = core::ptr::null();
        self.update_handle = 0;

        Ok(())
    }

    pub fn abort(&mut self) -> Result<(), EspError> {
        self.check_write()?;

        esp!(unsafe { esp_ota_abort(self.update_handle) })?;

        self.update_partition = core::ptr::null();
        self.update_handle = 0;

        Ok(())
    }

    fn check_write(&self) -> Result<(), EspError> {
        if !self.update_partition.is_null() {
            Ok(())
        } else {
            Err(EspError::from_infallible::<ESP_FAIL>())
        }
    }
}

#[derive(Debug)]
pub struct EspOta(());

impl EspOta {
    pub fn new() -> Result<Self, EspError> {
        let mut taken = TAKEN.lock();

        if *taken {
            return Err(EspError::from_infallible::<ESP_ERR_INVALID_STATE>());
        }

        *taken = true;

        Ok(Self(()))
    }

    pub fn get_boot_slot(&self) -> Result<Slot, EspError> {
        self.get_slot(unsafe { esp_ota_get_boot_partition().as_ref().unwrap() })
    }

    pub fn get_running_slot(&self) -> Result<Slot, EspError> {
        self.get_slot(unsafe { esp_ota_get_running_partition().as_ref().unwrap() })
    }

    pub fn get_update_slot(&self) -> Result<Slot, EspError> {
        self.get_slot(unsafe {
            esp_ota_get_next_update_partition(ptr::null())
                .as_ref()
                .unwrap()
        })
    }

    pub fn get_last_invalid_slot(&self) -> Result<Option<Slot>, EspError> {
        if let Some(partition) = unsafe { esp_ota_get_last_invalid_partition().as_ref() } {
            Ok(Some(self.get_slot(partition)?))
        } else {
            Ok(None)
        }
    }

    pub fn is_factory_reset_supported(&self) -> Result<bool, EspError> {
        self.get_factory_partition()
            .map(|factory| !factory.is_null())
    }

    pub fn factory_reset(&mut self) -> Result<(), EspError> {
        let factory = self.get_factory_partition()?;

        esp!(unsafe { esp_ota_set_boot_partition(factory) })?;

        Ok(())
    }

    pub fn initiate_update(&mut self) -> Result<EspOtaUpdate<'_>, EspError> {
        // This might return a null pointer in case no valid partition can be found.
        // We don't have to handle this error in here, as this will implicitly trigger an error
        // as soon as the null pointer is provided to `esp_ota_begin`.
        let partition = unsafe { esp_ota_get_next_update_partition(ptr::null()) };

        let mut handle: esp_ota_handle_t = Default::default();

        esp!(unsafe { esp_ota_begin(partition, OTA_SIZE_UNKNOWN as usize, &mut handle) })?;

        Ok(EspOtaUpdate {
            update_partition: partition,
            update_handle: handle,
            _data: PhantomData,
        })
    }

    pub fn mark_running_slot_valid(&mut self) -> Result<(), EspError> {
        Ok(esp!(unsafe { esp_ota_mark_app_valid_cancel_rollback() })?)
    }

    pub fn mark_running_slot_invalid_and_reboot(&mut self) -> EspError {
        if let Err(err) = esp!(unsafe { esp_ota_mark_app_invalid_rollback_and_reboot() }) {
            err
        } else {
            unreachable!()
        }
    }

    fn get_factory_partition(&self) -> Result<*const esp_partition_t, EspError> {
        let partition_iterator = unsafe {
            esp_partition_find(
                esp_partition_type_t_ESP_PARTITION_TYPE_APP,
                esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_APP_FACTORY,
                b"factory\0" as *const _ as *const _,
            )
        };

        if partition_iterator.is_null() {
            return Err(EspError::from_infallible::<ESP_ERR_NOT_SUPPORTED>());
        }

        let partition = unsafe { esp_partition_get(partition_iterator) };

        unsafe { esp_partition_iterator_release(partition_iterator) };

        Ok(partition)
    }

    fn get_slot(&self, partition: &esp_partition_t) -> Result<Slot, EspError> {
        Ok(Slot {
            label: unsafe { from_cstr_ptr(&partition.label as *const _ as *const _).into() },
            state: self.get_state(partition)?,
            firmware: self.get_firmware_info(partition)?,
        })
    }

    fn get_state(&self, partition: &esp_partition_t) -> Result<ota::SlotState, EspError> {
        let mut state: esp_ota_img_states_t = Default::default();

        let err =
            unsafe { esp_ota_get_state_partition(partition as *const _, &mut state as *mut _) };

        Ok(if err == ESP_ERR_NOT_FOUND {
            ota::SlotState::Unknown
        } else if err == ESP_ERR_NOT_SUPPORTED {
            ota::SlotState::Factory
        } else {
            esp!(err)?;

            #[allow(non_upper_case_globals)]
            match state {
                esp_ota_img_states_t_ESP_OTA_IMG_NEW
                | esp_ota_img_states_t_ESP_OTA_IMG_PENDING_VERIFY => ota::SlotState::Unverified,
                esp_ota_img_states_t_ESP_OTA_IMG_VALID => ota::SlotState::Valid,
                esp_ota_img_states_t_ESP_OTA_IMG_INVALID
                | esp_ota_img_states_t_ESP_OTA_IMG_ABORTED => ota::SlotState::Invalid,
                esp_ota_img_states_t_ESP_OTA_IMG_UNDEFINED => ota::SlotState::Unknown,
                _ => ota::SlotState::Unknown,
            }
        })
    }

    fn get_firmware_info(
        &self,
        partition: &esp_partition_t,
    ) -> Result<Option<ota::FirmwareInfo>, EspError> {
        let mut app_desc: esp_app_desc_t = Default::default();

        let err =
            unsafe { esp_ota_get_partition_description(partition as *const _, &mut app_desc) };

        Ok(if err == ESP_ERR_NOT_FOUND {
            None
        } else {
            esp!(err)?;

            Some(Newtype(&app_desc).into())
        })
    }
}

impl Drop for EspOta {
    fn drop(&mut self) {
        *TAKEN.lock() = false;

        info!("Dropped");
    }
}

impl io::ErrorType for EspOta {
    type Error = EspIOError;
}

impl ota::Ota for EspOta {
    type Update<'a> = EspOtaUpdate<'a> where Self: 'a;

    fn get_boot_slot(&self) -> Result<Slot, Self::Error> {
        EspOta::get_boot_slot(self).map_err(EspIOError)
    }

    fn get_running_slot(&self) -> Result<Slot, Self::Error> {
        EspOta::get_running_slot(self).map_err(EspIOError)
    }

    fn get_update_slot(&self) -> Result<Slot, Self::Error> {
        EspOta::get_update_slot(self).map_err(EspIOError)
    }

    fn is_factory_reset_supported(&self) -> Result<bool, Self::Error> {
        EspOta::is_factory_reset_supported(self).map_err(EspIOError)
    }

    fn factory_reset(&mut self) -> Result<(), Self::Error> {
        EspOta::factory_reset(self).map_err(EspIOError)
    }

    fn initiate_update(&mut self) -> Result<Self::Update<'_>, Self::Error> {
        EspOta::initiate_update(self).map_err(EspIOError)
    }

    fn mark_running_slot_valid(&mut self) -> Result<(), Self::Error> {
        EspOta::mark_running_slot_valid(self).map_err(EspIOError)
    }

    fn mark_running_slot_invalid_and_reboot(&mut self) -> Self::Error {
        EspIOError(EspOta::mark_running_slot_invalid_and_reboot(self))
    }
}

unsafe impl<'a> Send for EspOtaUpdate<'a> {}

impl<'a> io::ErrorType for EspOtaUpdate<'a> {
    type Error = EspIOError;
}

impl<'a> ota::OtaUpdate for EspOtaUpdate<'a> {
    fn complete(&mut self) -> Result<(), Self::Error> {
        EspOtaUpdate::complete(self)?;

        Ok(())
    }

    fn abort(&mut self) -> Result<(), Self::Error> {
        EspOtaUpdate::abort(self)?;

        Ok(())
    }
}

impl<'a> io::Write for EspOtaUpdate<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let size = EspOtaUpdate::write(self, buf)?;

        Ok(size)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        EspOtaUpdate::flush(self)?;

        Ok(())
    }
}
