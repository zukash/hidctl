use crate::args::CliError;

pub(crate) fn validate_length(actual: usize, expected: usize) -> Result<(), CliError> {
    if actual != expected {
        Err(CliError::new(
            2,
            format!("byte length is {actual}, expected {expected}"),
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn parse_bytes(input: &str) -> Result<Vec<u8>, CliError> {
    if input.trim().is_empty() {
        return Err(CliError::new(2, "--bytes cannot be empty"));
    }
    let tokens: Vec<_> = input
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .collect();
    if tokens.iter().any(|token| token.is_empty()) {
        return Err(CliError::new(2, "--bytes contains an empty value"));
    }
    // A bare numeric list is decimal; a list containing hex markers is hex.
    // This keeps both "17 255" and "11,ff" unambiguous at the command line.
    let hex_mode = tokens.iter().any(|token| {
        token.starts_with("0x")
            || token.starts_with("0X")
            || token
                .chars()
                .any(|character| character.is_ascii_alphabetic())
    });
    tokens
        .into_iter()
        .map(|token| {
            let value = parse_byte(token, hex_mode).map_err(|message| {
                CliError::new(2, format!("invalid byte '{token}': {message}"))
            })?;
            if value > u8::MAX as u64 {
                return Err(CliError::new(
                    2,
                    format!("byte '{token}' is outside 0..255"),
                ));
            }
            Ok(value as u8)
        })
        .collect()
}

fn parse_byte(value: &str, hex_mode: bool) -> Result<u64, String> {
    let (radix, digits) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map_or((if hex_mode { 16 } else { 10 }, value), |v| (16, v));
    if digits.is_empty() {
        return Err("empty number".into());
    }
    u64::from_str_radix(digits, radix).map_err(|_| "expected decimal or hexadecimal number".into())
}

pub(crate) fn parse_number(value: &str) -> Result<u64, String> {
    let (radix, digits) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .map_or((10, value), |v| (16, v));
    if digits.is_empty() {
        return Err("empty number".into());
    }
    u64::from_str_radix(digits, radix).map_err(|_| "expected decimal or hexadecimal number".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mixed_byte_formats() {
        assert_eq!(
            parse_bytes("11,ff,0a,1a,00").unwrap(),
            [0x11, 0xff, 0x0a, 0x1a, 0x00]
        );
        assert_eq!(parse_bytes("17 255 10 26 0").unwrap(), [17, 255, 10, 26, 0]);
    }

    #[test]
    fn rejects_empty_and_out_of_range_bytes() {
        assert!(parse_bytes("").is_err());
        assert!(parse_bytes("11,,22").is_err());
        assert!(parse_bytes("256").is_err());
    }

    #[test]
    fn enforces_report_length() {
        assert!(validate_length(3, 3).is_ok());
        assert!(validate_length(2, 3).is_err());
    }

    #[test]
    fn parses_hex_numbers() {
        assert_eq!(parse_number("0xff").unwrap(), 255);
        assert_eq!(parse_number("255").unwrap(), 255);
    }
}
