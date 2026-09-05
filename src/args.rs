use crate::bytes::parse_number;
use crate::device::Filter;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
pub(crate) struct CliError {
    pub(crate) code: u8,
    pub(crate) message: String,
}

impl CliError {
    pub(crate) fn new(code: u8, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub(crate) fn usage(message: impl Into<String>) -> CliError {
    CliError::new(
        2,
        format!("{}\ntry 'hidctl list' or see README.md", message.into()),
    )
}

#[derive(Default, Debug)]
pub(crate) struct Options {
    values: BTreeMap<String, String>,
    flags: BTreeSet<String>,
}

pub(crate) fn collect_options(args: Vec<String>) -> Result<Options, CliError> {
    let mut options = Options::default();
    let mut iterator = args.into_iter();
    while let Some(argument) = iterator.next() {
        let name = argument
            .strip_prefix("--")
            .ok_or_else(|| usage(format!("unexpected argument: {argument}")))?;
        if name == "json" {
            if !options.flags.insert(name.into()) {
                return Err(usage("duplicate --json"));
            }
            continue;
        }
        let value = iterator
            .next()
            .ok_or_else(|| usage(format!("missing value for --{name}")))?;
        if value.starts_with("--") {
            return Err(usage(format!("missing value for --{name}")));
        }
        if options.values.insert(name.into(), value).is_some() {
            return Err(usage(format!("duplicate --{name}")));
        }
    }
    Ok(options)
}

impl Options {
    pub(crate) fn ensure_allowed(&self, allowed: &[&str]) -> Result<(), CliError> {
        for name in self.values.keys().chain(self.flags.iter()) {
            if !allowed.contains(&name.as_str()) {
                return Err(usage(format!(
                    "option --{name} is not valid for this command"
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn required(&self, name: &str) -> Result<&str, CliError> {
        self.values
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| usage(format!("missing --{name}")))
    }

    pub(crate) fn flag(&self, name: &str) -> Result<bool, CliError> {
        if self.values.contains_key(name) {
            return Err(usage(format!("--{name} does not take a value")));
        }
        Ok(self.flags.contains(name))
    }

    fn optional(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    pub(crate) fn optional_usize(&self, name: &str) -> Result<Option<usize>, CliError> {
        self.optional(name)
            .map(|value| {
                value.parse().map_err(|_| {
                    CliError::new(2, format!("--{name} must be a non-negative integer"))
                })
            })
            .transpose()
    }

    pub(crate) fn optional_i32(&self, name: &str) -> Result<Option<i32>, CliError> {
        self.optional(name)
            .map(|value| {
                value
                    .parse()
                    .map_err(|_| CliError::new(2, format!("--{name} must be an integer")))
            })
            .transpose()
    }

    pub(crate) fn required_u8(&self, name: &str) -> Result<u8, CliError> {
        let value = parse_number(self.required(name)?)
            .map_err(|message| CliError::new(2, format!("--{name}: {message}")))?;
        u8::try_from(value).map_err(|_| CliError::new(2, format!("--{name} is outside 0..255")))
    }

    pub(crate) fn filter(&self) -> Result<Filter, CliError> {
        // Build every command's selector through the same numeric conversion path.
        let known = [
            "path",
            "serial",
            "vid",
            "pid",
            "usage-page",
            "usage",
            "json",
            "bytes",
            "length",
            "timeout",
            "report-id",
        ];
        for name in self.values.keys().chain(self.flags.iter()) {
            if !known.contains(&name.as_str()) {
                return Err(usage(format!("unknown option --{name}")));
            }
        }
        Ok(Filter {
            path: self.optional("path").map(str::to_owned),
            serial: self.optional("serial").map(str::to_owned),
            vid: self.optional_number("vid")?,
            pid: self.optional_number("pid")?,
            usage_page: self.optional_number("usage-page")?,
            usage: self.optional_number("usage")?,
        })
    }

    fn optional_number(&self, name: &str) -> Result<Option<u16>, CliError> {
        self.optional(name)
            .map(|value| {
                let number = parse_number(value)
                    .map_err(|message| CliError::new(2, format!("--{name}: {message}")))?;
                u16::try_from(number)
                    .map_err(|_| CliError::new(2, format!("--{name} is outside 0..65535")))
            })
            .transpose()
    }
}
