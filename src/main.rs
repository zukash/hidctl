mod args;
mod bytes;
mod device;
mod output;

use args::{collect_options, usage, CliError, Options};
use bytes::{parse_bytes, validate_length};
use device::{open_api, open_device, select_one, EXIT_ARGS};
use output::{print_bytes, print_info};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("hidctl: {}", error.message);
            ExitCode::from(error.code)
        }
    }
}

fn run() -> Result<(), CliError> {
    // Parse the command once, then keep each operation's validation local.
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| usage("missing command"))?;
    let options = collect_options(args.collect())?;

    match command.as_str() {
        "list" => list_devices(&options),
        "info" => {
            options.ensure_allowed(&["path", "serial", "vid", "pid", "usage-page", "usage"])?;
            let filter = options.filter()?;
            filter.validate()?;
            let api = open_api()?;
            let info = select_one(&api, &filter)?;
            print_info(info, false);
            Ok(())
        }
        "send-output" => send_report(&options, false),
        "send-feature" => send_report(&options, true),
        "read-input" => read_input(&options),
        "read-feature" => read_feature(&options),
        _ => Err(usage(format!("unknown command: {command}"))),
    }
}

fn list_devices(options: &Options) -> Result<(), CliError> {
    // Listing accepts partial filters so it can be used to discover selectors.
    options.ensure_allowed(&[
        "path",
        "serial",
        "vid",
        "pid",
        "usage-page",
        "usage",
        "json",
    ])?;
    let json = options.flag("json")?;
    let filter = options.filter()?;
    let api = open_api()?;
    let devices: Vec<_> = api
        .device_list()
        .filter(|info| filter.matches(info))
        .collect();
    if json {
        print!("[");
        for (index, info) in devices.iter().enumerate() {
            if index > 0 {
                print!(",");
            }
            output::print_info_json(info);
        }
        println!("]");
    } else {
        for info in devices {
            print_info(info, false);
            println!();
        }
    }
    Ok(())
}

fn send_report(options: &Options, feature: bool) -> Result<(), CliError> {
    // Reject malformed input before opening a device or causing side effects.
    options.ensure_allowed(&[
        "path",
        "serial",
        "vid",
        "pid",
        "usage-page",
        "usage",
        "bytes",
        "length",
    ])?;
    let filter = options.filter()?;
    filter.validate()?;
    let bytes = parse_bytes(options.required("bytes")?)?;
    if let Some(length) = options.optional_usize("length")? {
        validate_length(bytes.len(), length)?;
    }
    let api = open_api()?;
    let info = select_one(&api, &filter)?;
    print_info(info, true);
    let device = open_device(&api, info)?;
    let result = if feature {
        device.send_feature_report(&bytes).map(|_| ())
    } else {
        device.write(&bytes).map(|_| ())
    };
    result.map_err(|error| {
        device::report_error(
            if feature {
                "send-feature"
            } else {
                "send-output"
            },
            info,
            error,
        )
    })
}

fn read_input(options: &Options) -> Result<(), CliError> {
    options.ensure_allowed(&[
        "path",
        "serial",
        "vid",
        "pid",
        "usage-page",
        "usage",
        "timeout",
    ])?;
    let filter = options.filter()?;
    filter.validate()?;
    let timeout = options.optional_i32("timeout")?.unwrap_or(-1);
    if timeout < -1 {
        return Err(CliError::new(EXIT_ARGS, "--timeout must be -1 or greater"));
    }
    let api = open_api()?;
    let info = select_one(&api, &filter)?;
    print_info(info, true);
    let device = open_device(&api, info)?;
    let mut buffer = vec![0; 4096];
    let size = device
        .read_timeout(&mut buffer, timeout)
        .map_err(|error| device::report_error("read-input", info, error))?;
    print_bytes(&buffer[..size]);
    Ok(())
}

fn read_feature(options: &Options) -> Result<(), CliError> {
    options.ensure_allowed(&[
        "path",
        "serial",
        "vid",
        "pid",
        "usage-page",
        "usage",
        "report-id",
        "length",
    ])?;
    let filter = options.filter()?;
    filter.validate()?;
    let report_id = options.required_u8("report-id")?;
    let length = options.optional_usize("length")?.unwrap_or(256);
    if length == 0 {
        return Err(CliError::new(
            EXIT_ARGS,
            "--length must be greater than zero",
        ));
    }
    let api = open_api()?;
    let info = select_one(&api, &filter)?;
    print_info(info, true);
    let device = open_device(&api, info)?;
    // Feature reports include the requested report ID in the first byte.
    let mut buffer = vec![0; length];
    buffer[0] = report_id;
    let size = device
        .get_feature_report(&mut buffer)
        .map_err(|error| device::report_error("read-feature", info, error))?;
    print_bytes(&buffer[..size]);
    Ok(())
}
