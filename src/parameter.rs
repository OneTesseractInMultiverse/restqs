//! Query-string parameter decoding.

use crate::{ParserLimits, RqsError, RqsResult};

pub(crate) fn decode_parameters(query: &str, limits: ParserLimits) -> RqsResult<Vec<String>> {
    if query.len() > limits.max_query_bytes {
        return Err(RqsError::QueryTooLarge {
            max_bytes: limits.max_query_bytes,
        });
    }

    let mut parameters = Vec::new();
    for raw in query.split('&').filter(|parameter| !parameter.is_empty()) {
        if parameters.len() >= limits.max_parameters {
            return Err(RqsError::TooManyParameters {
                max_parameters: limits.max_parameters,
            });
        }
        parameters.push(decode_component(raw)?);
    }
    Ok(parameters)
}

fn decode_component(raw: &str) -> RqsResult<String> {
    let mut output = Vec::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' => {
                let Some(decoded) = decode_hex_byte(bytes, index) else {
                    return Err(RqsError::InvalidEncoding);
                };
                output.push(decoded);
                index += 3;
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(output).map_err(|_| RqsError::InvalidEncoding)
}

fn decode_hex_byte(bytes: &[u8], index: usize) -> Option<u8> {
    let high = *bytes.get(index + 1)?;
    let low = *bytes.get(index + 2)?;
    Some(hex_value(high)? * 16 + hex_value(low)?)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
