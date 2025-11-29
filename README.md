# Glint

Local-first observability for developers.

Glint is a lightweight OTLP collector with a modern UI. No Docker, no complex setup — just one binary.

## Install
```bash
# coming soon
curl -fsSL https://glint.dev/install.sh | sh
```

## Usage
```bash
# start collector + UI
# automatically detects your project and stores data in ~/.config/glint/<project>.db
glint serve

# open http://localhost:7070 for the bundled web UI
```

Point your OTLP exporter to:
- gRPC: `localhost:4317`
- HTTP: `localhost:4318`

```bash
# query traces (from current project)
glint traces

# query logs
glint logs

# show database info
glint info

# clean current project database
glint clean

# clean all databases
glint clean --all
```

## Development 
see in [Development](./DEVELOPMENT.md)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/ErickJ3/glint
cd glint

# Start the collector
# Database will be created at /home/<user>/.local/share/glint/<project-folder>.db
cargo run -p glint-cli -- serve

# In another terminal, send test traces
cargo run -p glint-cli --example send_traces

# Query traces
cargo run -p glint-cli -- traces

# Show database info
cargo run -p glint-cli -- info

# Clean database
cargo run -p glint-cli -- clean
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific package tests
cargo test -p glint-collector
cargo test -p glint-server
```

### Build

```bash
# Development build
cargo build

# Release build
cargo build --release

# The binary will be at target/release/glint
```

This produces `ui/dist`, which the server serves at `http://localhost:7070` (or the custom port you pass to `glint serve`).

## License

MIT
