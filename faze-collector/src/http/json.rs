//! OTLP/JSON quirk handling.
//!
//! The pbjson-generated serde impls follow canonical proto3 JSON, which
//! encodes `bytes` fields as base64. OTLP/JSON deviates for IDs: `trace_id`,
//! `span_id`, and `parent_span_id` are hex-encoded. This module rewrites the
//! hex IDs into base64 before deserialization so the generated impls accept
//! spec-compliant payloads.

use crate::convert::hex_to_bytes;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::Value;

/// Field names (both proto3 JSON casings) that OTLP/JSON encodes as hex.
const HEX_ID_KEYS: &[&str] = &[
    "traceId",
    "spanId",
    "parentSpanId",
    "trace_id",
    "span_id",
    "parent_span_id",
];

/// Rewrite hex-encoded ID strings into base64 in place.
///
/// Values that fail strict hex validation are left untouched, so the
/// downstream base64 decode reports the malformed field.
pub fn normalize_otlp_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                if HEX_ID_KEYS.contains(&key.as_str())
                    && let Value::String(s) = v
                {
                    if let Some(bytes) = hex_to_bytes(s) {
                        *v = Value::String(BASE64.encode(bytes));
                    }
                } else {
                    normalize_otlp_json(v);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(normalize_otlp_json),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalize_rewrites_hex_ids() {
        let mut value = json!({
            "resourceSpans": [{
                "scopeSpans": [{
                    "spans": [{
                        "traceId": "0102030405060708090a0b0c0d0e0f10",
                        "spanId": "0102030405060708",
                        "parentSpanId": "",
                        "name": "op"
                    }]
                }]
            }]
        });
        normalize_otlp_json(&mut value);

        let span = &value["resourceSpans"][0]["scopeSpans"][0]["spans"][0];
        assert_eq!(
            span["traceId"],
            BASE64.encode(hex_to_bytes("0102030405060708090a0b0c0d0e0f10").unwrap())
        );
        assert_eq!(
            span["spanId"],
            BASE64.encode(hex_to_bytes("0102030405060708").unwrap())
        );
        // Empty stays empty.
        assert_eq!(span["parentSpanId"], "");
        // Non-ID fields untouched.
        assert_eq!(span["name"], "op");
    }

    #[test]
    fn test_normalize_leaves_invalid_hex_untouched() {
        let mut value = json!({"traceId": "zz-not-hex"});
        normalize_otlp_json(&mut value);
        assert_eq!(value["traceId"], "zz-not-hex");
    }

    #[test]
    fn test_normalize_handles_snake_case_keys() {
        let mut value = json!({"trace_id": "abcd"});
        normalize_otlp_json(&mut value);
        assert_eq!(
            value["trace_id"],
            BASE64.encode(hex_to_bytes("abcd").unwrap())
        );
    }
}
