# Faze

> Local-first observability for developers.

Faze is a lightweight OTLP collector with an embedded web interface. It provides a simple way to collect, store, and visualize telemetry data without requiring Docker or complex infrastructure setup.

## A look

- Single binary with no external dependencies
- OTLP collector supporting both gRPC and HTTP protocols (partial)
- Embedded web UI for data visualization
- Project-based database management
- SQLite storage for traces, logs, and metrics

## Installation

### From Source

```bash
git clone https://github.com/ErickJ3/faze
cd faze
cargo build --release
```

The binary will be available at `target/release/faze`.

### Development Setup

If you're planning to contribute or hack on Faze, we recommend using Nix and Just to make your life easier.

**With Nix:**

We have a flake that sets up everything you need (Rust, protobuf, dependencies, the works):

```bash
nix develop
```

That's it. You're ready to go.

**Using Just:**

We use [Just](https://github.com/casey/just) as a command runner. Think of it as `make` but less painful. Once you're in the dev environment:

```bash
# See all available commands
just

# Run the server with hot reload
just dev-server

# Run the UI dev server
just dev-ui

# Run both server and UI together
just dev

# Build the UI and create a release binary
just build

# Run tests (Rust + UI)
just test

# Run UI tests with interactive interface
just test-ui

# Run UI tests with coverage report
just test-ui-coverage

# Run linters (Rust + UI)
just check
```

Just makes common tasks way more convenient than typing out full cargo commands every time.

## Usage

### Start the Collector

```bash
faze serve
```

This command:
- Starts the OTLP collector (gRPC on port 4317, HTTP on port 4318)
- Serves the web UI on http://localhost:7070
- Automatically detects your project and stores data in `~/.local/share/faze/<project>.db`

### Query Traces

```bash
faze traces
```

### Query Logs

```bash
faze logs
```

### DB Management

```bash
# Show database information
faze info

# Clean current project database
faze clean

# Clean all databases
faze clean --all
```

## OTLP

Configure your OTLP exporter to send telemetry to:
> good to know: we haven't fully implemented the protocol yet, and some things are still pending in HTTP.
- gRPC: `localhost:4317`
- HTTP: `localhost:4318`

## Storage

Faze stores telemetry data in SQLite databases located at:

- Linux/macOS: `~/.local/share/faze/<project>.db`
- Windows: `%LOCALAPPDATA%/faze/<project>.db`

Each project gets its own database, automatically detected from the current working directory.

## Requirements

- Rust 1.91+ (for building from source)
- Protobuf compiler (for building from source)

## License

MIT/Apache-2
