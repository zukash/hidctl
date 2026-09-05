use crate::args::CliError;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use std::fmt::Write as _;

pub(crate) const EXIT_ARGS: u8 = 2;

pub(crate) const EXIT_HID: u8 = 1;
const EXIT_NOT_FOUND: u8 = 3;
const EXIT_AMBIGUOUS: u8 = 4;
const EXIT_REPORT: u8 = 5;

#[derive(Default, Debug)]
pub(crate) struct Filter {
    pub(crate) path: Option<String>,
    pub(crate) serial: Option<String>,
    pub(crate) vid: Option<u16>,
    pub(crate) pid: Option<u16>,
    pub(crate) usage_page: Option<u16>,
    pub(crate) usage: Option<u16>,
}

impl Filter {
    pub(crate) fn validate(&self) -> Result<(), CliError> {
        if self.path.is_none()
            && self.serial.is_none()
            && (self.vid.is_none() || self.pid.is_none())
        {
            return Err(CliError::new(
                EXIT_ARGS,
                "device selector requires --path, --serial, or both --vid and --pid",
            ));
        }
        Ok(())
    }

    pub(crate) fn matches(&self, info: &DeviceInfo) -> bool {
        // Each supplied selector narrows the same candidate set.
        self.path
            .as_deref()
            .is_none_or(|value| info.path().to_string_lossy() == value)
            && self
                .serial
                .as_deref()
                .is_none_or(|value| info.serial_number() == Some(value))
            && self.vid.is_none_or(|value| info.vendor_id() == value)
            && self.pid.is_none_or(|value| info.product_id() == value)
            && self
                .usage_page
                .is_none_or(|value| info.usage_page() == value)
            && self.usage.is_none_or(|value| info.usage() == value)
    }
}

pub(crate) fn open_api() -> Result<HidApi, CliError> {
    HidApi::new()
        .map_err(|error| CliError::new(EXIT_HID, format!("HIDAPI initialization failed: {error}")))
}

pub(crate) fn select_one<'a>(api: &'a HidApi, filter: &Filter) -> Result<&'a DeviceInfo, CliError> {
    // Keep all matches so an incomplete selector cannot silently pick a device.
    let devices: Vec<_> = api
        .device_list()
        .filter(|info| filter.matches(info))
        .collect();
    match devices.as_slice() {
        [] => Err(CliError::new(
            EXIT_NOT_FOUND,
            format!("no HID device matched {}", filter_summary(filter)),
        )),
        [info] => Ok(info),
        _ => {
            let mut message = format!("multiple HID devices matched {}:\n", filter_summary(filter));
            for info in devices {
                let _ = writeln!(message, "  {}", device_summary(info));
            }
            Err(CliError::new(EXIT_AMBIGUOUS, message))
        }
    }
}

pub(crate) fn open_device(api: &HidApi, info: &DeviceInfo) -> Result<HidDevice, CliError> {
    // Open only the selected path; do not claim the whole HID manager.
    api.open_path(info.path()).map_err(|error| {
        CliError::new(
            EXIT_HID,
            format!(
                "failed to open {}: {error}; macOS may require Input Monitoring permission for this binary",
                info.path().to_string_lossy()
            ),
        )
    })
}

pub(crate) fn device_summary(info: &DeviceInfo) -> String {
    format!(
        "0x{:04x}:0x{:04x} usage 0x{:04x}/0x{:04x} {}",
        info.vendor_id(),
        info.product_id(),
        info.usage_page(),
        info.usage(),
        info.path().to_string_lossy()
    )
}

fn filter_summary(filter: &Filter) -> String {
    let mut parts = Vec::new();
    if let Some(value) = &filter.path {
        parts.push(format!("path={value}"));
    }
    if let Some(value) = &filter.serial {
        parts.push(format!("serial={value}"));
    }
    if let Some(value) = filter.vid {
        parts.push(format!("vid=0x{value:04x}"));
    }
    if let Some(value) = filter.pid {
        parts.push(format!("pid=0x{value:04x}"));
    }
    if let Some(value) = filter.usage_page {
        parts.push(format!("usage-page=0x{value:04x}"));
    }
    if let Some(value) = filter.usage {
        parts.push(format!("usage=0x{value:04x}"));
    }
    parts.join(", ")
}

pub(crate) fn report_error(
    operation: &str,
    info: &DeviceInfo,
    error: impl std::fmt::Display,
) -> CliError {
    CliError::new(
        EXIT_REPORT,
        format!("{operation} failed for {}: {error}", device_summary(info)),
    )
}
