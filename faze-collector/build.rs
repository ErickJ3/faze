#![allow(missing_docs)]

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("otlp_descriptor.bin");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(
            &[
                "proto/opentelemetry/proto/collector/trace/v1/trace_service.proto",
                "proto/opentelemetry/proto/collector/logs/v1/logs_service.proto",
                "proto/opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
                "proto/google/rpc/status.proto",
            ],
            &["proto"],
        )?;

    // Serde impls for OTLP/JSON support on the HTTP collector. Unknown fields
    // are skipped (matching protobuf wire semantics) so payloads from newer
    // SDKs never fail to decode.
    let descriptor_set = std::fs::read(descriptor_path)?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .ignore_unknown_fields()
        .build(&[".opentelemetry", ".google.rpc"])?;

    Ok(())
}
