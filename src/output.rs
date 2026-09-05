use crate::device::device_summary;
use hidapi::DeviceInfo;
use std::fmt::Write as _;

pub(crate) fn print_info(info: &DeviceInfo, report_context: bool) {
    if report_context {
        println!("Target: {}", device_summary(info));
    } else {
        println!("Vendor ID:      0x{:04x}", info.vendor_id());
        println!("Product ID:     0x{:04x}", info.product_id());
        println!(
            "Manufacturer:   {}",
            info.manufacturer_string().unwrap_or("")
        );
        println!("Product:        {}", info.product_string().unwrap_or(""));
        println!("Serial Number:  {}", info.serial_number().unwrap_or(""));
        println!("Usage Page:     0x{:04x}", info.usage_page());
        println!("Usage:          0x{:04x}", info.usage());
        println!("Bus Type:       {:?}", info.bus_type());
        println!("Path:           {}", info.path().to_string_lossy());
    }
}

pub(crate) fn print_info_json(info: &DeviceInfo) {
    // Avoid adding a JSON dependency for this fixed, small output shape.
    print!(
        "{{\"vendor_id\":{},\"product_id\":{},\"manufacturer\":{},\"product\":{},\"serial_number\":{},\"usage_page\":{},\"usage\":{},\"bus_type\":{},\"path\":{}}}",
        info.vendor_id(),
        info.product_id(),
        json_string(info.manufacturer_string().unwrap_or("")),
        json_string(info.product_string().unwrap_or("")),
        json_string(info.serial_number().unwrap_or("")),
        info.usage_page(),
        info.usage(),
        json_string(&format!("{:?}", info.bus_type())),
        json_string(&info.path().to_string_lossy())
    );
}

pub(crate) fn print_bytes(bytes: &[u8]) {
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 {
            print!(" ");
        }
        print!("{:02x}", byte);
    }
    println!();
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}
