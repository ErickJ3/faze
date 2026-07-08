//! Content negotiation and body decompression for the OTLP/HTTP collector.

use axum::body::Bytes;
use axum::http::{HeaderMap, header};
use std::io::Read;

/// Wire encoding of an OTLP/HTTP request/response pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    /// `application/x-protobuf`
    Protobuf,
    /// `application/json` (OTLP/JSON)
    Json,
}

impl Encoding {
    /// The content-type header value for this encoding.
    pub const fn content_type(self) -> &'static str {
        match self {
            Self::Protobuf => "application/x-protobuf",
            Self::Json => "application/json",
        }
    }
}

/// Cap on decompressed request size, guarding against gzip bombs.
const MAX_DECOMPRESSED_BYTES: u64 = 64 * 1024 * 1024;

/// Resolve the request encoding from `Content-Type`.
///
/// OTLP/HTTP requires `application/x-protobuf` or `application/json`;
/// anything else (including a missing header) is unsupported.
pub fn negotiate(headers: &HeaderMap) -> Option<Encoding> {
    let content_type = headers.get(header::CONTENT_TYPE)?.to_str().ok()?;
    // Strip parameters such as "; charset=utf-8".
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    match mime.as_str() {
        "application/x-protobuf" => Some(Encoding::Protobuf),
        "application/json" => Some(Encoding::Json),
        _ => None,
    }
}

/// Decompression failure modes surfaced to the handler.
pub enum DecompressError {
    /// `Content-Encoding` other than gzip/identity.
    UnsupportedEncoding(String),
    /// Body did not decompress cleanly.
    Corrupt,
    /// Decompressed payload exceeded [`MAX_DECOMPRESSED_BYTES`].
    TooLarge,
}

/// Apply `Content-Encoding` to the raw request body.
pub fn decompress(headers: &HeaderMap, body: Bytes) -> Result<Bytes, DecompressError> {
    let encoding = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("identity")
        .trim()
        .to_ascii_lowercase();

    match encoding.as_str() {
        "identity" | "" => Ok(body),
        "gzip" => {
            let mut decoder =
                flate2::read::GzDecoder::new(&body[..]).take(MAX_DECOMPRESSED_BYTES + 1);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|_| DecompressError::Corrupt)?;
            if decompressed.len() as u64 > MAX_DECOMPRESSED_BYTES {
                return Err(DecompressError::TooLarge);
            }
            Ok(Bytes::from(decompressed))
        }
        other => Err(DecompressError::UnsupportedEncoding(other.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn headers_with(name: header::HeaderName, value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(name, value.parse().unwrap());
        headers
    }

    #[test]
    fn test_negotiate() {
        assert_eq!(
            negotiate(&headers_with(
                header::CONTENT_TYPE,
                "application/x-protobuf"
            )),
            Some(Encoding::Protobuf)
        );
        assert_eq!(
            negotiate(&headers_with(
                header::CONTENT_TYPE,
                "application/json; charset=utf-8"
            )),
            Some(Encoding::Json)
        );
        assert_eq!(
            negotiate(&headers_with(header::CONTENT_TYPE, "text/plain")),
            None
        );
        assert_eq!(negotiate(&HeaderMap::new()), None);
    }

    #[test]
    fn test_decompress_gzip_roundtrip() {
        let payload = b"hello otlp";
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(payload).unwrap();
        let compressed = encoder.finish().unwrap();

        let headers = headers_with(header::CONTENT_ENCODING, "gzip");
        let result = decompress(&headers, Bytes::from(compressed)).ok().unwrap();
        assert_eq!(&result[..], payload);
    }

    #[test]
    fn test_decompress_identity_passthrough() {
        let body = Bytes::from_static(b"raw");
        let result = decompress(&HeaderMap::new(), body.clone()).ok().unwrap();
        assert_eq!(result, body);
    }

    #[test]
    fn test_decompress_rejects_unknown_encoding() {
        let headers = headers_with(header::CONTENT_ENCODING, "br");
        assert!(matches!(
            decompress(&headers, Bytes::new()),
            Err(DecompressError::UnsupportedEncoding(_))
        ));
    }

    #[test]
    fn test_decompress_rejects_corrupt_gzip() {
        let headers = headers_with(header::CONTENT_ENCODING, "gzip");
        assert!(matches!(
            decompress(&headers, Bytes::from_static(b"not gzip")),
            Err(DecompressError::Corrupt)
        ));
    }
}
