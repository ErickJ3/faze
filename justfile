default:
    @just --list

dev-server:
    RUST_LOG=info cargo watch -x 'run -p glint-cli -- serve'

dev-ui:
    cd ui && bun dev

dev:
    just dev-server &
    just dev-ui

build-ui:
    cd ui && bun run build

build: build-ui
    cargo build --release -p glint-cli

test:
    cargo test --workspace

check:
    cargo clippy --workspace
    cargo fmt --check

clean:
    cargo clean
    rm -rf ui/node_modules ui/dist
